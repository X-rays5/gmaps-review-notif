use poise::serenity_prelude::Client;
use tracing_subscriber::FmtSubscriber;

mod discord;
mod crawler;

#[tokio::main]
async fn main() {
    load_env().await;

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing_log::LogTracer::init().expect("failed to init logger");

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let discord_client = discord::builder::build(token).await;

    match discord_client {
        Ok(mut client) => {
            run_discord_client(&mut client).await;
        }
        Err(e) => eprintln!("Failed to initialize Discord client: {}", e),
    }
}

async fn load_env() {
    match dotenvy::dotenv_override() {
        Ok(_) => (),
        Err(e) => eprintln!("Failed to load .env file: {}", e),
    }
}

async fn run_discord_client(client: &mut Client) {
    if let Err(e) = client.start_autosharded().await {
        eprintln!("Discord client error: {}", e);
    }
}