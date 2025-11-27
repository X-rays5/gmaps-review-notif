use std::sync::LazyLock;

pub struct Config {
    pub star_text: String,
    pub fetch_reviews_on_startup: bool,
    pub new_review_fetch_interval: String,
    pub discord_token: String,
    pub database_url: String,
    pub review_age_limit_hours: i64,
    pub chrome_path: Option<String>,
    pub browser_timeout_ms: u64,
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| Config {
    star_text: std::env::var("STAR_TEXT").unwrap_or_else(|_| "⭐".to_string()),
    fetch_reviews_on_startup: std::env::var("FETCH_REVIEWS_ON_STARTUP")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true",
    new_review_fetch_interval: std::env::var("NEW_REVIEW_FETCH_INTERVAL")
        .unwrap_or_else(|_| "0 0 */6 * * *".to_string()),
    discord_token: std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"),
    database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    review_age_limit_hours: std::env::var("REVIEW_AGE_LIMIT_HOURS")
        .unwrap_or_else(|_| "24".to_string())
        .parse()
        .unwrap_or(24),
    chrome_path: std::env::var("CHROME_PATH").ok(),
    browser_timeout_ms: std::env::var("BROWSER_TIMEOUT_MS")
        .unwrap_or_else(|_| "10000".to_string())
        .parse()
        .unwrap_or(10000),
});

pub fn get_config() -> &'static Config {
    &CONFIG
}
