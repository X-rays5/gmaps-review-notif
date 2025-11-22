extern crate core;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use poise::serenity_prelude::Client;
use tracing_subscriber::FmtSubscriber;
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::background::worker;
use crate::config::get_config;
use crate::provider::db::DbProvider;

mod discord;
mod crawler;
mod schema;
mod models;
mod provider;
mod config;
mod background;
mod utility;

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

    let discord_client = discord::builder::build(config::get_config().discord_token.clone()).await;

    schedule_background_review_check().await;

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

async fn schedule_background_review_check() {
    let scheduler = JobScheduler::new().await.unwrap();

    if get_config().fetch_reviews_on_startup {
        tokio::task::spawn(async move {
            tracing::info!("Running startup review check...");
            worker::check_for_new_reviews();
        });
    }

    let job = match Job::new(get_config().new_review_fetch_interval.clone(), |_uuid, _l| {
        worker::check_for_new_reviews();
    }) {
        Ok(j) => j,
        Err(e) => {
            tracing::error!("Failed to create scheduled job with schedule '{}': '{}'", get_config().new_review_fetch_interval.clone(), e);
            return;
        }
    };

    scheduler.add(job).await.unwrap();
    scheduler.start().await.unwrap();
}

async fn run_discord_client(client: &mut Client) {
    if let Err(e) = client.start_autosharded().await {
        tracing::error!("Discord client error: {}", e);
    }
}