use anyhow::Error;

pub mod follow;
pub mod followed;
pub mod latest;
pub mod lookup;

type CommandCtx<'a, U> = poise::Context<'a, U, Error>;

async fn ack<U: Sync>(ctx: &CommandCtx<'_, U>) {
    match ctx.defer().await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Failed to acknowledge command: {}", e);
        }
    }
}
