use crate::models::ReviewWithUser;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};

pub fn get_review_embed(review_with_user: &ReviewWithUser, original: bool) -> CreateEmbed {
    let review_body = if original {
        match &review_with_user.review.original_text {
            Some(text) => text.as_str(),
            None => review_with_user.review.text.as_str(),
        }
    } else {
        review_with_user.review.text.as_str()
    };

    CreateEmbed::default()
        .title(review_with_user.review.place_name.clone())
        .field(
            "Stars",
            crate::config::get_config()
                .star_text
                .repeat(review_with_user.review.stars.try_into().unwrap()),
            false,
        )
        .author(CreateEmbedAuthor::new(review_with_user.user.name.clone()))
        .timestamp(review_with_user.review.found_at.and_utc())
        .description(review_body)
        .footer(CreateEmbedFooter::new(format!(
            "Due to caching, this review may be up to {} hours old.",
            crate::config::get_config().review_age_limit_hours
        )))
}
