use once_cell::sync::Lazy;

pub struct Config {
    pub star_text: String,
}

static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config {
        star_text: std::env::var("STAR_TEXT").unwrap_or_else(|_| "⭐".to_string())
    }
});

pub fn get_config() -> &'static Config {
    &CONFIG
}