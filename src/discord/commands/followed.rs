use crate::discord::commands::{ack, CommandCtx};
use crate::provider;
use anyhow::Result;

/// Gets a list of users being followed in the current channel.
#[poise::command(
    slash_command,
    rename = "followed",
    default_member_permissions = "MANAGE_WEBHOOKS",
    required_bot_permissions = "MANAGE_WEBHOOKS"
)]
pub async fn followed_command<U: Sync>(ctx: CommandCtx<'_, U>) -> Result<()> {
    ack(&ctx).await;

    let users = match provider::following::get_users_followed_in_channel(
        ctx.channel_id().get().to_string(),
    ) {
        Ok(u) => u,
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to retrieve followed users: {}", e))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    if users.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content("ℹ️ No users are currently being followed in this channel.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let mut content = String::from("📋 **Followed Users in this Channel:**\n");
    for user in users {
        content.push_str(&format!("- {} (`{}`)\n", user.name, user.gmaps_id));
    }
    ctx.send(
        poise::CreateReply::default()
            .content(content)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
