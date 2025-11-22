use anyhow::{Error, Result};
use poise::CreateReply;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};
use crate::provider::*;

type Context<'a, U> = poise::Context<'a, U, Error>;

#[poise::command(
    slash_command,
    rename = "follow",
    required_permissions = "MANAGE_WEBHOOKS"
)]
pub async fn follow_user<U: Sync>(
    ctx: Context<'_, U>,
    #[description = "The ID of the user to follow"]
    id: String,
    #[description = "Enable or disable following"]
    enabled: Option<bool>,
) -> Result<()> {
    ctx.defer().await?;
    ctx.reply("follow_user").await?;

    Ok(())
}

#[poise::command(
    slash_command,
    rename = "lookup",
    required_permissions = "SEND_MESSAGES"
)]
pub async fn lookup_user<U: Sync>(
    ctx: Context<'_, U>,
    #[description = "The ID of the user to look up"]
    id: String,
) -> Result<()> {
    ctx.defer().await?;

    match user::get_user_from_gmaps_id(id) {
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

#[poise::command(
    slash_command,
    rename = "latest",
    required_permissions = "SEND_MESSAGES"
)]
pub async fn latest_review<U: Sync>(
    ctx: Context<'_, U>,
    #[description = "The ID of the user to get the latest review from"]
    id: String,
    original: Option<bool>,
) -> Result<()> {
    ctx.defer().await?;

    match review::get_latest_review_for_user(id.as_str()) {
        Some(review_with_user) => {
            let review_body = if original.unwrap_or(false) {
                match &review_with_user.review.review_original_text {
                    Some(text) => text.as_str(),
                    None => review_with_user.review.review_text.as_str(),
                }
            } else {
                review_with_user.review.review_text.as_str()
            };
            let embed = CreateEmbed::default()
                .title(review_with_user.review.place_name)
                .field("Stars", crate::config::get_config().star_text.repeat(review_with_user.review.stars.try_into()?), false)
                .author(CreateEmbedAuthor::new(review_with_user.user.name))
                .timestamp(review_with_user.review.found_at.and_utc())
                .description(review_body)
                .footer(CreateEmbedFooter::new("Due to caching this might not be the most recent review"));


            ctx.send(CreateReply::default().embed(embed).ephemeral(false)).await?;
        }
        None => {
            ctx.send(
                CreateReply::default()
                    .content("❌ No reviews found for the specified user.")
                    .ephemeral(true)
            ).await?;
        }
    }

    Ok(())
}