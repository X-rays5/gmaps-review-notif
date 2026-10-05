use crate::background::worker;
use crate::discord::commands::{ack, CommandCtx};
use crate::provider::*;
use crate::utility::snowflake;
use anyhow::Result;
use poise::serenity_prelude::CreateWebhook;

/// Start or stop following a user in the current channel.
#[poise::command(
    slash_command,
    rename = "follow",
    default_member_permissions = "MANAGE_WEBHOOKS",
    required_bot_permissions = "MANAGE_WEBHOOKS"
)]
pub async fn follow_user<U: Sync>(
    ctx: CommandCtx<'_, U>,
    #[description = "The ID of the user to follow"] id: String,
    #[description = "Enable or disable following"] enabled: bool,
    original: Option<bool>,
) -> Result<()> {
    ack(&ctx).await;

    handle_follow_switch(id, enabled, original.unwrap_or(true), ctx).await;

    Ok(())
}

async fn handle_follow_switch<U: Sync>(
    gmaps_id: String,
    enable: bool,
    original: bool,
    ctx: CommandCtx<'_, U>,
) {
    let channel_id = ctx.channel_id().to_string();
    let user_id = match user::gmaps_user_id_to_db_id(gmaps_id.as_ref()) {
        Some(id) => id,
        None => {
            tracing::warn!(
                gmaps_id = %gmaps_id,
                channel_id = %channel_id,
                "Follow command referenced a Google Maps user that is not known to us"
            );
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("❌ Unable to retrieve specified user")
                        .ephemeral(true),
                )
                .await;
            return;
        }
    };

    let is_followed = following::is_user_followed_in_channel(user_id, channel_id.clone());

    if enable {
        handle_enable(is_followed, user_id, gmaps_id.as_str(), original, ctx).await;
    } else {
        handle_disable(is_followed, user_id, gmaps_id.as_str(), ctx).await;
    }
}

async fn handle_enable(
    is_followed: bool,
    user_id: i32,
    gmaps_id: &str,
    original: bool,
    ctx: CommandCtx<'_, impl Sync>,
) {
    if is_followed {
        tracing::info!(
            db_user_id = user_id,
            gmaps_id = %gmaps_id,
            channel_id = %ctx.channel_id(),
            "Follow request ignored: user is already followed in this channel"
        );
        let _ = ctx
            .send(
                poise::CreateReply::default()
                    .content("⚠️ User is already being followed in this channel")
                    .ephemeral(true),
            )
            .await;
        return;
    }

    let channel = match ctx.guild_channel().await {
        Some(c) => c,
        None => {
            tracing::warn!(
                db_user_id = user_id,
                gmaps_id = %gmaps_id,
                channel_id = %ctx.channel_id(),
                "Follow request ignored: channel information is unavailable"
            );
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("❌ Unable to retrieve channel information")
                        .ephemeral(true),
                )
                .await;
            return;
        }
    };

    let webhook = match channel
        .create_webhook(
            &ctx.http(),
            CreateWebhook::new(format!("Google Maps Reviews - {}", user_id)),
        )
        .await
    {
        Ok(w) => w,
        Err(e) => {
            tracing::error!(
                db_user_id = user_id,
                gmaps_id = %gmaps_id,
                channel_id = %ctx.channel_id(),
                error = %e,
                "Failed to create webhook for new follow"
            );
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to create webhook: {}", e))
                        .ephemeral(true),
                )
                .await;
            return;
        }
    };
    tracing::info!(
        db_user_id = user_id,
        gmaps_id = %gmaps_id,
        channel_id = %ctx.channel_id(),
        webhook_id = webhook.id.get(),
        "Created webhook for new follow"
    );

    match following::follow_user_in_channel(
        user_id,
        ctx.channel_id().to_string(),
        original,
        webhook.id.to_string(),
    ) {
        Ok(following) => {
            tracing::info!(
                db_user_id = user_id,
                gmaps_id = %gmaps_id,
                channel_id = %following.channel_id,
                webhook_id = %following.webhook_id,
                original_text = following.original_text,
                "Now following user"
            );

            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("✅ Now following user in this channel")
                        .ephemeral(true),
                )
                .await;

            tokio::task::spawn(
                async move { worker::channel_started_following_user(following).await },
            );
        }
        Err(e) => {
            tracing::error!(
                db_user_id = user_id,
                gmaps_id = %gmaps_id,
                channel_id = %ctx.channel_id(),
                webhook_id = webhook.id.get(),
                error = %e,
                "Failed to save follow record; the created webhook was left in place"
            );
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to follow user: {}", e))
                        .ephemeral(true),
                )
                .await;
        }
    }
}

async fn handle_disable(is_followed: bool, user_id: i32, gmaps_id: &str, ctx: CommandCtx<'_, impl Sync>) {
    if !is_followed {
        tracing::info!(
            db_user_id = user_id,
            gmaps_id = %gmaps_id,
            channel_id = %ctx.channel_id(),
            "Unfollow request ignored: user is not followed in this channel"
        );
        let _ = ctx
            .send(
                poise::CreateReply::default()
                    .content("⚠️ User is not being followed in this channel")
                    .ephemeral(true),
            )
            .await;
        return;
    }
    match following::unfollow_user_in_channel(user_id, ctx.channel_id().to_string()) {
        Ok(following) => {
            match snowflake::webhook_id(following.webhook_id.as_str()) {
                Some(webhook_id) => {
                    match ctx
                        .http()
                        .delete_webhook(webhook_id, Some("User was unfollowed"))
                        .await
                    {
                        Ok(_) => tracing::info!(
                            db_user_id = user_id,
                            gmaps_id = %gmaps_id,
                            channel_id = %following.channel_id,
                            webhook_id = %following.webhook_id,
                            "Deleted webhook for unfollowed user"
                        ),
                        Err(e) => {
                            tracing::warn!(
                                db_user_id = user_id,
                                gmaps_id = %gmaps_id,
                                channel_id = %following.channel_id,
                                webhook_id = %following.webhook_id,
                                error = %e,
                                "Failed to delete webhook for unfollowed user"
                            );
                            let _ = ctx
                                .send(
                                    poise::CreateReply::default()
                                        .content(format!("⚠️ Failed to delete webhook: {}", e))
                                        .ephemeral(true),
                                )
                                .await;
                        }
                    }
                }
                None => tracing::warn!(
                    db_user_id = user_id,
                    gmaps_id = %gmaps_id,
                    channel_id = %following.channel_id,
                    webhook_id = %following.webhook_id,
                    "Stored webhook ID is invalid; nothing to delete"
                ),
            }

            tracing::info!(
                db_user_id = user_id,
                gmaps_id = %gmaps_id,
                channel_id = %following.channel_id,
                "Unfollowed user"
            );

            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("✅ Unfollowed user in this channel")
                        .ephemeral(true),
                )
                .await;
        }
        Err(e) => {
            tracing::error!(
                db_user_id = user_id,
                gmaps_id = %gmaps_id,
                channel_id = %ctx.channel_id(),
                error = %e,
                "Failed to unfollow user"
            );
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to unfollow user: {}", e))
                        .ephemeral(true),
                )
                .await;
        }
    }
}
