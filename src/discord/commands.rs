use anyhow::{Error, Result};
use crate::crawler::pages::{review, user};

type Context<'a, U> = poise::Context<'a, U, Error>;

static STAR_EMOJI: &str = "<:starrating:1433928079805517874>";

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

    match user::get_user_from_id(&id) {
        Ok(user) => {
            let embed = poise::serenity_prelude::CreateEmbed::default()
                .title("User Lookup")
                .field("Name", &user.name, true)
                .field("ID", &user.id, true)
                .color(0x4A90E2);

            ctx.send(
                poise::CreateReply::default()
                    .embed(embed)
            ).await?;
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to look up user: {}", e))
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

    match review::get_latest_review_for_user(&id) {
        Ok(review) => {
            let embed = poise::serenity_prelude::CreateEmbed::default()
                .title("Latest Review")
                .field("Place", &review.place_name, false)
                .field("Rating", STAR_EMOJI.repeat(review.star_rating as usize), true)
                .field("Review", &review.text, false)
                .color(0x34A853);

            ctx.send(
                poise::CreateReply::default()
                    .embed(embed)
            ).await?;
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to get latest review: {}", e))
                    .ephemeral(true)
            ).await?;
        }
    }

    Ok(())
}