use crate::discord::commands::{ack, CommandCtx};
use crate::provider::*;
use anyhow::Result;
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;

/// Look up a user by their Google Maps ID.
#[poise::command(
    slash_command,
    rename = "lookup",
    default_member_permissions = "SEND_MESSAGES",
    required_bot_permissions = "EMBED_LINKS"
)]
pub async fn lookup_user<U: Sync>(
    ctx: CommandCtx<'_, U>,
    #[description = "The ID of the user to look up"]
    id: String,
) -> Result<()> {
    ack(&ctx).await;

    match user::get_user_from_gmaps_id(id.as_str()) {
        Ok(user) => {
            let embed = CreateEmbed::default()
                .title("User Lookup")
                .field("Name", user.name, false)
                .field("GMaps ID", user.gmaps_id, false)
                .color(0x00FF00);
            ctx.send(CreateReply::default().embed(embed).ephemeral(true)).await?;
        }
        Err(e) => {
            ctx.send(
                CreateReply::default()
                    .content(format!("❌ Failed to find user: {}", e))
                    .ephemeral(true)
            ).await?;
        }
    }

    Ok(())
}