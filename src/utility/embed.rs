use crate::models::ReviewWithUser;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};

use crate::crawler::pages::user::GMAPS_USER_URL;

pub fn get_review_embed(review_with_user: &ReviewWithUser, original: bool) -> CreateEmbed {
    let review_body = select_review_body(review_with_user, original);

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

    let valid_pictures = collect_valid_pictures(&review_with_user.review.pictures);
    embed = embed.description(build_review_description(review_body, &valid_pictures));

    embed
}

fn select_review_body<'a>(review_with_user: &'a ReviewWithUser, original: bool) -> &'a str {
    if original {
        review_with_user
            .review
            .original_text
            .as_deref()
            .unwrap_or(review_with_user.review.text.as_str())
    } else {
        review_with_user.review.text.as_str()
    }
}

fn collect_valid_pictures(pictures: &serde_json::Value) -> Vec<&str> {
    pictures
        .as_array()
        .map(|pictures_arr| {
            pictures_arr
                .iter()
                .filter_map(|pic| pic.as_str())
                .filter(|pic| !pic.trim().is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn build_review_description(review_body: &str, valid_pictures: &[&str]) -> String {
    if valid_pictures.is_empty() {
        return review_body.to_string();
    }

    let mut description = format!("{review_body}\n\n\n");
    for (idx, pic) in valid_pictures.iter().enumerate() {
        description.push_str(format!("[Picture {}]({})\n", idx + 1, pic).as_str());
    }
    description
}

#[cfg(test)]
mod tests {
    use super::{build_review_description, collect_valid_pictures, select_review_body};
    use crate::models::{Review, ReviewWithUser, User};
    use chrono::Utc;
    use serde_json::json;

    fn sample_review_with_user(original_text: Option<&str>) -> ReviewWithUser {
        ReviewWithUser {
            user: User {
                id: 42,
                gmaps_id: "gmaps-42".to_string(),
                name: "Alice".to_string(),
            },
            review: Review {
                id: 7,
                place_name: "Cafe".to_string(),
                text: "Translated text".to_string(),
                original_text: original_text.map(str::to_string),
                stars: 5,
                user_id: 42,
                found_at: Utc::now().naive_utc(),
                link_en: Some("https://example.com/review".to_string()),
                pictures: json!([]),
            },
        }
    }

    #[test]
    fn select_review_body_prefers_original_when_requested() {
        let review_with_user = sample_review_with_user(Some("Original text"));
        assert_eq!(select_review_body(&review_with_user, true), "Original text");
    }

    #[test]
    fn select_review_body_falls_back_to_translated_text() {
        let review_with_user = sample_review_with_user(None);
        assert_eq!(select_review_body(&review_with_user, true), "Translated text");
        assert_eq!(select_review_body(&review_with_user, false), "Translated text");
    }

    #[test]
    fn collect_valid_pictures_keeps_only_non_empty_strings() {
        let pictures = json!(["https://img/1", "   ", null, 1, "https://img/2"]);
        assert_eq!(collect_valid_pictures(&pictures), vec!["https://img/1", "https://img/2"]);
    }

    #[test]
    fn build_review_description_without_pictures_is_plain_text() {
        assert_eq!(build_review_description("Body", &[]), "Body");
    }

    #[test]
    fn build_review_description_with_pictures_adds_links() {
        let description = build_review_description("Body", &["https://img/1", "https://img/2"]);
        assert_eq!(
            description,
            "Body\n\n\n[Picture 1](https://img/1)\n[Picture 2](https://img/2)\n"
        );
    }
}
