use anyhow::Result;
use reqwest::Client;
use reqwest::Url;
use serde::Deserialize;
use std::sync::OnceLock;
use std::time::Duration;

static SHORTENER_URL: &str = "https://s.scheenen.dev/shorten";

#[derive(Deserialize)]
struct ShortenResponse {
    url: String,
}

fn shortener_client() -> Result<&'static Client> {
    static SHORTENER_CLIENT: OnceLock<Result<Client, String>> = OnceLock::new();

    match SHORTENER_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())
    }) {
        Ok(client) => Ok(client),
        Err(message) => Err(anyhow::anyhow!("Failed to build the URL shortener HTTP client: {message}")),
    }
}

pub async fn shorten_url(url: &Url) -> Result<String> {
    let client = shortener_client()?;

    let resp = match client.post(SHORTENER_URL).body(url.as_str().to_owned()).send().await {
        Ok(resp) => resp,
        Err(err) => {
            return Err(anyhow::anyhow!("Failed to send request to URL shortener: {err}"));
        }
    };

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("Request failed: {}", resp.status()));
    }

    let resp_json: ShortenResponse = match resp.json().await {
        Ok(json) => json,
        Err(err) => {
            return Err(anyhow::anyhow!("Failed to parse shorten response json: {err}"));
        }
    };

    Ok(format!("https://{}", resp_json.url))
}
