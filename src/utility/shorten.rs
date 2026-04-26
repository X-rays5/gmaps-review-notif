use anyhow::Result;
use reqwest::Client;
use reqwest::Url;
use serde::Deserialize;
use std::sync::OnceLock;
use std::time::Duration;

static SHORTENER_URL: &str = "https://s.scheenen.dev/shorten";
static SHORTENER_CLIENT: OnceLock<Client> = OnceLock::new();

#[derive(Deserialize)]
struct ShortenResponse {
    url: String,
}

fn shortener_client() -> &'static Client {
    SHORTENER_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build shortener HTTP client")
    })
}

pub async fn shorten_url(url: &Url) -> Result<String> {
    let client = shortener_client();

    let resp = match client.post(SHORTENER_URL).body(url.as_str().to_owned()).send().await {
        Ok(resp) => resp,
        Err(err) => {
            tracing::error!("Failed to send request to URL shortener: {}", err);
            return Err(anyhow::anyhow!("Failed to send request to URL shortener: {err}"));
        }
    };

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("Request failed: {}", resp.status()));
    }

    let resp_json: ShortenResponse = match resp.json().await {
        Ok(json) => json,
        Err(err) => {
            tracing::error!("Failed to parse shorten response json: {}", err);
            return Err(anyhow::anyhow!("Failed to parse shorten response json: {err}"));
        }
    };

    Ok(format!("https://{}", resp_json.url))
}