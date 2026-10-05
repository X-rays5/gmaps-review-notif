use crate::config::get_config;
use crate::models::{Following, ReviewWithUser, User};
use crate::provider::following;
use crate::provider::following::get_followed_users_with_old_reviews;
use crate::utility::snowflake;
use crate::{provider, utility};
use poise::serenity_prelude as serenity;

pub async fn channel_started_following_user(following: Following) {
    let Some(review) = provider::review::get_latest_review_for_user(following.followed_user_id) else {
        tracing::info!(
            db_user_id = following.followed_user_id,
            channel_id = %following.channel_id,
            "No reviews found for newly followed user"
        );
        return;
    };

    notify_new_review(following, review).await;
}

pub fn check_for_new_reviews() {
    let started = std::time::Instant::now();
    tracing::info!("Starting review check");

    let users = match get_followed_users_with_old_reviews() {
        Ok(users) => users,
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch followed users with old reviews");
            return;
        }
    };

    // `-1` marks a count that could not be read (the provider logs the underlying error).
    let followed_users = following::get_amount_of_users_followed().unwrap_or(-1);
    let due_users = users.len();

    let (new_reviews, failures) = process_outdated_user_reviews(users);

    tracing::info!(
        followed_users,
        due_users,
        new_reviews,
        failures,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "Review check finished"
    );
}

fn process_outdated_user_reviews(users: Vec<User>) -> (usize, usize) {
    let mut new_reviews = 0usize;
    let mut failures = 0usize;

    for user in users {
        let Some(review) = provider::review::check_for_new_review(&user) else {
            continue;
        };

        let followers = match following::get_followers_of_user(user.id) {
            Ok(follows) => follows,
            Err(e) => {
                tracing::error!(db_user_id = user.id, gmaps_id = %user.gmaps_id, error = %e, "Failed to get followers of user");
                failures += 1;
                continue;
            }
        };

        new_reviews += 1;
        tracing::info!(
            db_user_id = user.id,
            gmaps_id = %user.gmaps_id,
            follower_count = followers.len(),
            "Notifying followers about new review"
        );

        for follower in followers {
            let review = review.clone();
            tokio::task::spawn(async move { notify_new_review(follower, review).await });
        }
    }

    (new_reviews, failures)
}

async fn notify_new_review(following: Following, review: ReviewWithUser) {
    tracing::info!(
        db_user_id = review.user.id,
        gmaps_id = %review.user.gmaps_id,
        channel_id = %following.channel_id,
        review_id = review.review.id,
        "Sending new review notification"
    );

    let http = serenity::Http::new(get_config().discord_token.as_str());
    let stored_webhook_id = snowflake::webhook_id(following.webhook_id.as_str());
    let Some(webhook_id) = ensure_webhook_exists(
        stored_webhook_id,
        following.channel_id.as_str(),
        &http,
    )
        .await else {
        tracing::error!(
            db_user_id = review.user.id,
            gmaps_id = %review.user.gmaps_id,
            channel_id = %following.channel_id,
            "Failed to ensure webhook exists"
        );
        return;
    };

    if stored_webhook_id != Some(webhook_id) {
        let new_webhook_id = webhook_id.get().to_string();
        match following::update_webhook(new_webhook_id.as_str(), following.channel_id.as_str()) {
            Ok(()) => tracing::info!(
                channel_id = %following.channel_id,
                webhook_id = %new_webhook_id,
                "Updated webhook ID for channel"
            ),
            Err(e) => tracing::error!(
                channel_id = %following.channel_id,
                webhook_id = %new_webhook_id,
                error = %e,
                "Failed to update webhook ID for channel"
            ),
        }
    }

    let webhook = match serenity::Webhook::from_id(&http, webhook_id).await {
        Ok(wh) => wh,
        Err(e) => {
            tracing::error!(
                channel_id = %following.channel_id,
                webhook_id = webhook_id.get(),
                error = %e,
                "Failed to fetch webhook by ID"
            );
            return;
        }
    };

    let current_user = match http.get_current_user().await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!(error = %e, "Failed to get current bot user");
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
        Ok(_) => tracing::info!(
            db_user_id = review.user.id,
            gmaps_id = %review.user.gmaps_id,
            channel_id = %following.channel_id,
            review_id = review.review.id,
            "Sent new review notification"
        ),
        Err(e) => tracing::error!(
            db_user_id = review.user.id,
            gmaps_id = %review.user.gmaps_id,
            channel_id = %following.channel_id,
            error = %e,
            "Failed to send webhook message"
        ),
    }
}

/// Returns the webhook to post through, reusing the stored one when it still exists and creating a
/// replacement otherwise. A stored ID that cannot be parsed is treated as missing.
async fn ensure_webhook_exists(
    stored_webhook_id: Option<serenity::WebhookId>,
    channel: &str,
    http: &serenity::Http,
) -> Option<serenity::WebhookId> {
    if let Some(webhook_id) = stored_webhook_id {
        match http.get_webhook(webhook_id).await {
            Ok(_) => return Some(webhook_id),
            Err(e) => {
                if let serenity::Error::Http(http_err) = &e
                    && let serenity::HttpError::UnsuccessfulRequest(resp) = http_err
                    && resp.status_code == 403
                {
                    tracing::error!(
                        channel_id = %channel,
                        "Missing permissions to access or create webhook in channel"
                    );
                    return None;
                }
            }
        }
    }

    let channel_id = snowflake::channel_id(channel)?;

    match http.create_webhook(channel_id, &(), None).await {
        Ok(webhook) => {
            tracing::info!(channel_id = %channel, webhook_id = webhook.id.get(), "Created new webhook for channel");
            Some(webhook.id)
        }
        Err(e) => {
            tracing::error!(channel_id = %channel, error = %e, "Failed to create webhook");
            None
        }
    }
}
