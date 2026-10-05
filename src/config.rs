use std::sync::LazyLock;

/// Upper bound for `REVIEW_AGE_LIMIT_HOURS`. A year is far beyond any sane setting and keeps the
/// `chrono` arithmetic that uses it clear of the values where it would overflow.
const MAX_REVIEW_AGE_LIMIT_HOURS: i64 = 24 * 365;

pub struct Config {
    pub star_text: String,
    pub fetch_reviews_on_startup: bool,
    pub new_review_fetch_interval: String,
    pub discord_token: String,
    pub database_url: String,
    pub review_age_limit_hours: i64,
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| Config {
    star_text: std::env::var("STAR_TEXT").unwrap_or_else(|_| "⭐".to_string()),
    fetch_reviews_on_startup: std::env::var("FETCH_REVIEWS_ON_STARTUP")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        == "true",
    new_review_fetch_interval: std::env::var("NEW_REVIEW_FETCH_INTERVAL")
        .unwrap_or_else(|_| "0 0 */6 * * *".to_string()),
    discord_token: std::env::var("DISCORD_TOKEN").unwrap_or_default(),
    database_url: std::env::var("DATABASE_URL").unwrap_or_default(),
    review_age_limit_hours: std::env::var("REVIEW_AGE_LIMIT_HOURS")
        .ok()
        .and_then(|raw| raw.parse::<i64>().ok())
        .filter(|hours| (1..=MAX_REVIEW_AGE_LIMIT_HOURS).contains(hours))
        .unwrap_or(24),
});

impl Config {
    /// Variables that hold no value but are required to run.
    ///
    /// Missing configuration is reported once at startup instead of panicking the first time a
    /// value is used.
    pub fn missing_required_values(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.discord_token.is_empty() {
            missing.push("DISCORD_TOKEN");
        }
        if self.database_url.is_empty() {
            missing.push("DATABASE_URL");
        }
        missing
    }
}

pub fn get_config() -> &'static Config {
    &CONFIG
}
