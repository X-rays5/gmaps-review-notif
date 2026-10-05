// Tests may panic freely; the lints below keep the production paths panic-free.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing
    )
)]

extern crate core;

use crate::background::worker;
use crate::config::get_config;
use crate::provider::db::DbProvider;
use anyhow::Result;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::process::ExitCode;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing_subscriber::FmtSubscriber;

mod background;
mod config;
mod crawler;
mod discord;
mod models;
mod provider;
mod schema;
mod utility;

pub const DIESEL_MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[tokio::main]
async fn main() -> ExitCode {
    let env_result = load_env();
    init_tracing();
    if let Err(e) = env_result {
        tracing::warn!(error = %e, "Failed to load the .env file; relying on the process environment");
    }

    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "Fatal error, shutting down");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let missing = get_config().missing_required_values();
    if !missing.is_empty() {
        return Err(anyhow::anyhow!(
            "Required environment variables are not set: {}",
            missing.join(", ")
        ));
    }

    init_db().await?;

    tokio::task::spawn(async move {
        if let Err(e) = schedule_review_checks().await {
            tracing::error!(error = %e, "Review check scheduler stopped; no further checks will run");
        }
    });

    let mut client = discord::builder::build(get_config().discord_token.clone()).await?;
    client.start_autosharded().await?;

    Ok(())
}

fn load_env() -> Result<(), dotenvy::Error> {
    dotenvy::dotenv_override().map(|_| ())
}

fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    // Logging that is not available is not a reason to abort, but it must be said out loud.
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set the tracing subscriber: {e}");
    }
    if let Err(e) = tracing_log::LogTracer::init() {
        eprintln!("Failed to initialize the log tracer: {e}");
    }
}

async fn init_db() -> Result<()> {
    let mut conn = DbProvider::global().get_connection()?;

    conn.run_pending_migrations(DIESEL_MIGRATIONS)
        .map_err(|e| anyhow::anyhow!("Failed to run pending database migrations: {e}"))?;

    Ok(())
}

async fn schedule_review_checks() -> Result<()> {
    let scheduler = JobScheduler::new().await?;

    if get_config().fetch_reviews_on_startup {
        tracing::info!(trigger = "startup", "Triggering review check");
        worker::check_for_new_reviews();
    }

    let interval = get_config().new_review_fetch_interval.clone();
    let job = Job::new(interval, |_uuid, _l| {
        tracing::info!(trigger = "schedule", "Triggering review check");
        worker::check_for_new_reviews();
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    Ok(())
}
