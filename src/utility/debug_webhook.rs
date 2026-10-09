//! Debug hook: posts a screenshot of the crawled page to a Discord webhook.
//!
//! A notification only shows the review we parsed, which is not enough to tell whether the crawler
//! read the review it meant to read. Setting `DEBUG_WEBHOOK_URL` makes every crawl post there, so
//! the page the parser saw can be compared with what Google shows to a human.
//!
//! It is a debugging aid, so nothing here is allowed to fail a crawl: callers log and carry on.

use crate::config::get_config;
use anyhow::Result;
use reqwest::multipart::{Form, Part};
use std::sync::OnceLock;
use std::time::Duration;

const DEBUG_WEBHOOK_TIMEOUT: Duration = Duration::from_secs(30);
const SCREENSHOT_FILE_NAME: &str = "review.png";

/// Whether a debug webhook is configured. Callers check this before paying for a screenshot.
pub fn is_enabled() -> bool {
    get_config().debug_webhook_url.is_some()
}

fn client() -> Result<&'static reqwest::Client> {
    static CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();

    match CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(DEBUG_WEBHOOK_TIMEOUT)
            .build()
            .map_err(|e| e.to_string())
    }) {
        Ok(client) => Ok(client),
        Err(message) => Err(anyhow::anyhow!(
            "Failed to build the debug webhook HTTP client: {message}"
        )),
    }
}

/// Posts `screenshot` to the configured debug webhook, under a message describing what the crawler
/// was looking at. Does nothing when no webhook is configured.
pub async fn send_screenshot(context: &str, screenshot: Vec<u8>) -> Result<()> {
    let Some(webhook_url) = get_config().debug_webhook_url.as_deref() else {
        return Ok(());
    };

    post_screenshot(webhook_url, context, screenshot).await
}

/// The part of [`send_screenshot`] that talks to the webhook, with the destination passed in so it
/// can be pointed at a test server.
async fn post_screenshot(webhook_url: &str, context: &str, screenshot: Vec<u8>) -> Result<()> {
    let payload = serde_json::json!({ "content": context }).to_string();
    let form = Form::new().text("payload_json", payload).part(
        "files[0]",
        Part::bytes(screenshot)
            .file_name(SCREENSHOT_FILE_NAME)
            .mime_str("image/png")?,
    );

    let response = client()?
        .post(webhook_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            // `reqwest::Error` writes the request URL into its message, and the webhook URL is the
            // secret itself, so callers must never be handed an error that carries it.
            anyhow::anyhow!("Failed to send the debug screenshot: {}", e.without_url())
        })?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Debug webhook responded with {}",
            response.status()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::post_screenshot;
    use std::io::{Read, Write};

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfake-image-bytes";

    #[tokio::test]
    async fn post_screenshot_posts_the_message_and_the_image_multipart_encoded() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind a local port");
        let address = listener.local_addr().expect("read the bound address");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept the request");
            stream
                .set_read_timeout(Some(std::time::Duration::from_millis(500)))
                .expect("set a read timeout");

            // Read the request until the client stops sending; the multipart body has no length
            // this side can predict.
            let mut request = Vec::new();
            let mut chunk = [0u8; 4096];
            while let Ok(read) = stream.read(&mut chunk) {
                request.extend_from_slice(&chunk[..read]);
            }

            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .expect("write the response");
            request
        });

        post_screenshot(
            &format!("http://{address}/webhook"),
            "Crawled review for Rick (109698308359600722911)",
            PNG.to_vec(),
        )
        .await
        .expect("the webhook accepts the request");

        let request = server.join().expect("the test server finishes");
        let text = String::from_utf8_lossy(&request);

        assert!(text.contains("multipart/form-data"));
        assert!(text.contains("payload_json"));
        assert!(text.contains("Crawled review for Rick"));
        assert!(text.contains("filename=\"review.png\""));
        assert!(text.contains("image/png"));
        assert!(
            request.windows(PNG.len()).any(|window| window == PNG),
            "the screenshot bytes are part of the request"
        );
    }

    #[tokio::test]
    async fn post_screenshot_reports_a_rejected_request() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind a local port");
        let address = listener.local_addr().expect("read the bound address");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept the request");
            let mut chunk = [0u8; 4096];
            let _ = stream.read(&mut chunk);
            stream
                .write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n")
                .expect("write the response");
        });

        let result = post_screenshot(&format!("http://{address}/webhook"), "context", PNG.to_vec()).await;

        assert!(result.is_err());
        server.join().expect("the test server finishes");
    }

    #[tokio::test]
    async fn post_screenshot_does_not_put_the_webhook_url_in_the_error() {
        // Nothing listens on port 1, so the request fails before it reaches anywhere.
        let result = post_screenshot(
            "http://127.0.0.1:1/webhook/secret-token",
            "context",
            PNG.to_vec(),
        )
        .await;

        let message = result.expect_err("the request cannot succeed").to_string();
        assert!(
            !message.contains("secret-token") && !message.contains("127.0.0.1"),
            "the webhook URL must not reach the logs: {message}"
        );
    }
}
