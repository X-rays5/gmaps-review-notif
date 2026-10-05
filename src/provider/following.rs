use crate::config::get_config;
use crate::models::{Following, User};
use crate::provider::db::get_connection;
use crate::schema::following;
use crate::schema::reviews;
use crate::schema::users;
use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;


pub fn get_amount_of_users_followed() -> Result<i64> {
    let mut conn = match get_connection() {
        Some(c) => c,
        None => {
            return Err(anyhow::anyhow!("Failed to get DB connection"));
        }
    };

    match following::table
        .select(following::followed_user_id)
        .distinct()
        .count()
        .first::<i64>(&mut conn)
    {
        Ok(count) => Ok(count),
        Err(e) => {
            tracing::error!(error = %e, "Failed to count followed users");
            Err(anyhow::anyhow!("Database query error: {e}"))
        }
    }
}

pub fn get_followed_users_with_old_reviews() -> Result<Vec<User>> {
    let mut conn = match get_connection() {
        Some(c) => c,
        None => {
            return Err(anyhow::anyhow!("Failed to get DB connection"));
        }
    };

    let age_limit_hours = get_config().review_age_limit_hours;
    let age_limit_duration = chrono::Duration::hours(age_limit_hours);
    let cutoff_time = (Utc::now() - age_limit_duration).naive_utc();

    match following::table
        .inner_join(users::table.on(users::id.eq(following::followed_user_id)))
        .left_join(reviews::table.on(reviews::user_id.eq(users::id)))
        .filter(
            reviews::found_at
                .lt(cutoff_time)
                .or(reviews::found_at.is_null()),
        )
        .select(users::all_columns)
        .distinct()
        .load::<User>(&mut conn)
    {
        Ok(users) => Ok(users),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load followed users with old reviews");
            Err(anyhow::anyhow!("Database query error: {}", e))
        }
    }
}

pub fn get_followers_of_user(user_id: i32) -> Result<Vec<Following>> {
    let mut conn = match get_connection() {
        Some(c) => c,
        None => {
            return Err(anyhow::anyhow!("Failed to get DB connection"));
        }
    };

    match following::table
        .filter(following::followed_user_id.eq(user_id))
        .load::<Following>(&mut conn)
    {
        Ok(followings) => Ok(followings),
        Err(e) => {
            tracing::error!(db_user_id = user_id, error = %e, "Failed to load followers of user");
            Err(anyhow::anyhow!("Database query error: {}", e))
        }
    }
}

pub fn get_users_followed_in_channel(channel: String) -> Result<Vec<User>> {
    let mut conn = match get_connection() {
        Some(c) => c,
        None => {
            return Err(anyhow::anyhow!("Failed to get DB connection"));
        }
    };

    match following::table
        .inner_join(users::table.on(users::id.eq(following::followed_user_id)))
        .filter(following::channel_id.eq(channel.as_str()))
        .select(users::all_columns)
        .load::<User>(&mut conn)
    {
        Ok(users) => Ok(users),
        Err(e) => {
            tracing::error!(channel_id = %channel, error = %e, "Failed to load users followed in channel");
            Err(anyhow::anyhow!("Database query error: {}", e))
        }
    }
}

pub fn is_user_followed_in_channel(user_id: i32, channel: String) -> bool {
    let Some(mut conn) = get_connection() else {
        return false;
    };

    match following::table
        .filter(following::followed_user_id.eq(user_id))
        .filter(following::channel_id.eq(channel.as_str()))
        .first::<Following>(&mut conn)
    {
        Ok(_) => true,
        Err(diesel::result::Error::NotFound) => false,
        Err(e) => {
            tracing::error!(db_user_id = user_id, channel_id = %channel, error = %e, "Failed to check if user is followed in channel");
            false
        }
    }
}

pub fn follow_user_in_channel(
    user_id: i32,
    channel: String,
    original_text: bool,
    webhook: String,
) -> Result<Following> {
    let mut conn = match get_connection() {
        Some(c) => c,
        None => {
            return Err(anyhow::anyhow!("Failed to get DB connection"));
        }
    };

    let new_following = crate::models::NewFollowing {
        followed_user_id: user_id,
        channel_id: channel,
        original_text,
        webhook_id: webhook,
    };

    match diesel::insert_into(following::table)
        .values(&new_following)
        .get_result::<Following>(&mut conn)
    {
        Ok(following) => Ok(following),
        Err(e) => {
            tracing::error!(
                db_user_id = user_id,
                channel_id = %new_following.channel_id,
                error = %e,
                "Failed to follow user"
            );
            Err(anyhow::anyhow!("Database insert error: {}", e))
        }
    }
}

pub fn unfollow_user_in_channel(user_id: i32, channel: String) -> Result<Following> {
    let mut conn = match get_connection() {
        Some(c) => c,
        None => {
            return Err(anyhow::anyhow!("Failed to get DB connection"));
        }
    };

    match diesel::delete(
        following::table
            .filter(following::followed_user_id.eq(user_id))
            .filter(following::channel_id.eq(channel.as_str())),
    )
        .get_result::<Following>(&mut conn)
    {
        Ok(deleted) => Ok(deleted),
        Err(e) => {
            tracing::error!(db_user_id = user_id, channel_id = %channel, error = %e, "Failed to unfollow user");
            Err(anyhow::anyhow!("Database delete error: {}", e))
        }
    }
}

pub fn update_webhook(webhook: &str, channel_id: &str) -> Result<()> {
    let mut conn = match get_connection() {
        Some(c) => c,
        None => {
            return Err(anyhow::anyhow!("Failed to get DB connection"));
        }
    };

    match diesel::update(following::table.filter(following::channel_id.eq(channel_id)))
        .set(following::webhook_id.eq(webhook))
        .execute(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!(channel_id = %channel_id, error = %e, "Failed to update webhook");
            Err(anyhow::anyhow!("Database update error: {}", e))
        }
    }
}
