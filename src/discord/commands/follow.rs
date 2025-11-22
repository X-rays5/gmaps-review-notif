use crate::background::worker;
use crate::discord::commands::{ack, CommandCtx};
use crate::provider::*;
use anyhow::Result;
use poise::serenity_prelude::{CreateWebhook, WebhookId};

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
    let user_id = match user::gmaps_user_id_to_db_id(gmaps_id.as_ref()) {
        Some(id) => id,
        None => {
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

    let is_followed = following::is_user_followed_in_channel(user_id, ctx.channel_id().to_string());

    if enable {
        handle_enable(is_followed, user_id, original, ctx).await;
    } else {
        handle_disable(is_followed, user_id, ctx).await;
    }
}

async fn handle_enable(
    is_followed: bool,
    user_id: i32,
    original: bool,
    ctx: CommandCtx<'_, impl Sync>,
) {
    if is_followed {
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

    match following::follow_user_in_channel(
        user_id,
        ctx.channel_id().to_string(),
        original,
        webhook.id.to_string(),
    ) {
        Ok(following) => {
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

async fn handle_disable(is_followed: bool, user_id: i32, ctx: CommandCtx<'_, impl Sync>) {
    if !is_followed {
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
            match ctx
                .http()
                .delete_webhook(
                    WebhookId::new(following.webhook_id.parse().unwrap()),
                    Some("User was unfollowed"),
                )
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    let _ = ctx
                        .send(
                            poise::CreateReply::default()
                                .content(format!("⚠️ Failed to delete webhook: {}", e))
                                .ephemeral(true),
                        )
                        .await;
                }
            }

            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("✅ Unfollowed user in this channel")
                        .ephemeral(true),
                )
                .await;
        }
        Err(e) => {
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
