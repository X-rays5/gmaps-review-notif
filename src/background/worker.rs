use crate::config::get_config;
use crate::models::{Following, ReviewWithUser, User};
use crate::provider::following;
use crate::provider::following::get_followed_users_with_old_reviews;
use crate::{provider, utility};
use poise::serenity_prelude as serenity;

pub async fn channel_started_following_user(following: Following) {
    let Some(review) = provider::review::get_latest_review_for_user(following.followed_user_id) else {
        tracing::info!(
                "No reviews found for newly followed user with id: {}",
                following.followed_user_id
            );
        return;
    };

    notify_new_review(following, review).await;
}

pub fn check_for_new_reviews() {
    match get_followed_users_with_old_reviews() {
        Ok(users) => {
            tracing::info!(
                "Found '{}'/'{}' followed users with reviews past age limit",
                users.len(),
                following::get_amount_of_users_followed().unwrap()
            );
            process_outdated_user_reviews(users);
        }
        Err(e) => {
            tracing::error!("Failed to fetch followed users with old reviews: {}", e);
        }
    }
}

fn process_outdated_user_reviews(users: Vec<User>) {
    for user in users {
        let Some(review) = provider::review::get_new_review(user.id) else {
            tracing::info!(
                    "No new reviews found for followed user with id: {}",
                    user.id
                );
            continue;
        };

        let followers = match following::get_followers_of_user(user.id) {
            Ok(follows) => follows,
            Err(e) => {
                tracing::error!("Failed to get followings for user id {}: {}", user.id, e);
                continue;
            }
        };

        for follower in followers {
            let review = review.clone();
            tokio::task::spawn(async move { notify_new_review(follower, review).await });
        }
    }
}

async fn notify_new_review(following: Following, review: ReviewWithUser) {
    let http = serenity::Http::new(get_config().discord_token.as_str());
    let Some(webhook_id) = ensure_webhook_exists(
        following.webhook_id.as_str(),
        following.channel_id.as_str(),
        &http,
    )
        .await else {
        tracing::error!(
                "Failed to ensure webhook exists for channel_id: {}",
                following.channel_id
            );
        return;
    };

    if webhook_id != following.webhook_id {
        match following::update_webhook(webhook_id.as_str(), following.channel_id.as_str()) {
            Ok(()) => tracing::info!(
                "Updated webhook ID for channel_id: {}",
                following.channel_id
            ),
            Err(e) => tracing::error!(
                "Failed to update webhook ID for channel_id: {}: {}",
                following.channel_id,
                e
            ),
        }
    }

    let webhook = match serenity::Webhook::from_id(
        &http,
        serenity::WebhookId::new(webhook_id.parse().unwrap()),
    )
        .await
    {
        Ok(wh) => wh,
        Err(e) => {
            tracing::error!("Failed to fetch webhook by ID {}: {}", webhook_id, e);
            return;
        }
    };

    let current_user = match http.get_current_user().await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to get current bot user: {}", e);
            return;
        }
    };

    let webhook_message = serenity::ExecuteWebhook::new()
        .username(current_user.name.clone())
        .avatar_url(current_user.avatar_url().unwrap_or_default())
        .embed(utility::embed::get_review_embed(
            &review,
            following.original_text,
        ));
    match webhook.execute(&http, false, webhook_message).await {
        Ok(_) => tracing::debug!(
            "Successfully sent new review notification: gmaps='{}', channel='{}'",
            review.user.gmaps_id,
            following.channel_id
        ),
        Err(e) => tracing::error!("Failed to send webhook message: {}", e),
    }
}

async fn ensure_webhook_exists(
    webhook: &str,
    channel_id: &str,
    http: &serenity::Http,
) -> Option<String> {
    let webhook_id = serenity::WebhookId::new(webhook.parse().unwrap());

    match http.get_webhook(webhook_id).await {
        Ok(_) => Some(webhook.to_string()),
        Err(e) => {
            if let serenity::Error::Http(http_err) = &e {
                if let serenity::HttpError::UnsuccessfulRequest(resp) = http_err {
                    if resp.status_code == 403 {
                        tracing::error!(
                            "Missing permissions to access or create webhook in channel {}",
                            channel_id
                        );
                        return None;
                    }
                }
            }

            match http
                .create_webhook(
                    serenity::ChannelId::new(channel_id.parse().unwrap()),
                    &(),
                    None,
                )
                .await
            {
                Ok(webhook) => Some(webhook.id.to_string()),
                Err(e) => {
                    tracing::error!("Failed to create webhook: {}", e);
                    None
                }
            }
        }
    }
}
