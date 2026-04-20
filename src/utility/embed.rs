use crate::models::ReviewWithUser;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};

use crate::crawler::pages::user::GMAPS_USER_URL;

pub fn get_review_embed(review_with_user: &ReviewWithUser, original: bool) -> CreateEmbed {
    let review_body = if original {
        match &review_with_user.review.original_text {
            Some(text) => text.as_str(),
            None => review_with_user.review.text.as_str(),
        }
    } else {
        review_with_user.review.text.as_str()
    };

    let mut embed = CreateEmbed::default()
        .title(review_with_user.review.place_name.clone())
        .field(
            "Stars",
            crate::config::get_config()
                .star_text
                .repeat(review_with_user.review.stars.try_into().unwrap()),
            false,
        )
        .author(CreateEmbedAuthor::new(review_with_user.user.name.clone()).url(GMAPS_USER_URL.replace("{}", review_with_user.user.gmaps_id.as_str())))
        .timestamp(review_with_user.review.found_at.and_utc())
        .footer(CreateEmbedFooter::new(format!(
            "Due to caching, this review may be up to {} hours old.",
            crate::config::get_config().review_age_limit_hours
        )));

    if review_with_user.review.link_en.is_some() {
        embed = embed.url(review_with_user.review.link_en.clone().unwrap());
    }

    // Handle pictures as JSON array
    if let Some(pictures_arr) = review_with_user.review.pictures.as_array() {
        if pictures_arr.is_empty() {
            embed = embed.description(review_body);
        } else {
            let mut description = review_body.to_string();
            description.push_str("\n\n\n");
            for idx in 0..pictures_arr.len() {
                let pic = &pictures_arr[idx];
                description.push_str(format!("[Picture {}]({})\n", idx + 1, pic.as_str().unwrap_or("")).as_str());
            }
            embed = embed.description(description);
        }
    } else {
        embed = embed.description(review_body);
    }

    embed
}
