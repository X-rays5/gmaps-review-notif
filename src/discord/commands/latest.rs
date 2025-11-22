use crate::discord::commands::{ack, CommandCtx};
use crate::provider::*;
use crate::utility;
use anyhow::Result;
use poise::CreateReply;

/// Get the most recent review from a specified user.
#[poise::command(
    slash_command,
    rename = "latest",
    default_member_permissions = "SEND_MESSAGES"
)]
pub async fn latest_review<U: Sync>(
    ctx: CommandCtx<'_, U>,
    #[description = "The ID of the user to get the latest review from"] id: String,
    original: Option<bool>,
) -> Result<()> {
    ack(&ctx).await;

    match review::get_latest_review_for_user_gmaps_id(id.as_str()) {
        Some(review_with_user) => {
            ctx.send(
                CreateReply::default()
                    .embed(utility::embed::get_review_embed(
                        &review_with_user,
                        original.unwrap_or(true),
                    ))
                    .ephemeral(false),
            )
            .await?;
        }
        None => {
            ctx.send(
                CreateReply::default()
                    .content("❌ No reviews found for the specified user.")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
