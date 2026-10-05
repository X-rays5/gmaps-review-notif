use poise::serenity_prelude as serenity;
use std::num::NonZeroU64;

/// Parses a Discord ID that we hold as a string (a channel or webhook ID read from the database).
///
/// Discord IDs are non-zero `u64`s, so an empty, zero or malformed value means the stored data is
/// unusable. That is logged and reported as absent rather than taking the task down.
fn parse_snowflake(kind: &str, raw: &str) -> Option<NonZeroU64> {
    match raw.parse::<NonZeroU64>() {
        Ok(id) => Some(id),
        Err(e) => {
            tracing::error!(kind, id = %raw, error = %e, "Stored Discord ID is not a valid snowflake");
            None
        }
    }
}

pub fn channel_id(raw: &str) -> Option<serenity::ChannelId> {
    parse_snowflake("channel", raw).map(serenity::ChannelId::from)
}

pub fn webhook_id(raw: &str) -> Option<serenity::WebhookId> {
    parse_snowflake("webhook", raw).map(serenity::WebhookId::from)
}
