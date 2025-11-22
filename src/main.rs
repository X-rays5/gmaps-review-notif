extern crate core;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use poise::serenity_prelude::Client;
use tracing_subscriber::FmtSubscriber;
use crate::provider::db::DbProvider;

mod discord;
mod crawler;
mod schema;
mod models;
mod provider;
mod config;

pub const DIESEL_MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[tokio::main]
async fn main() {
    load_env().await;

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing_log::LogTracer::init().expect("failed to init logger");

    init_db().await;

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

async fn init_db() {
    let mut conn = DbProvider::global()
        .get_connection()
        .expect("connect failed");

    conn.run_pending_migrations(DIESEL_MIGRATIONS)
        .expect("migration failed");
}

async fn run_discord_client(client: &mut Client) {
    if let Err(e) = client.start_autosharded().await {
        eprintln!("Discord client error: {}", e);
    }
}