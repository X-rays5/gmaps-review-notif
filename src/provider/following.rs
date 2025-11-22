use crate::config::get_config;
use crate::models::{Following, User};
use crate::provider::db::DbConnection;
use crate::schema::following;
use crate::schema::reviews;
use crate::schema::users;
use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;

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
            tracing::error!("Failed to load followed users with old reviews: {}", e);
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
            tracing::error!("Failed to load followings for user {}: {}", user_id, e);
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
        .filter(following::channel_id.eq(channel))
        .select(users::all_columns)
        .load::<User>(&mut conn)
    {
        Ok(users) => Ok(users),
        Err(e) => {
            tracing::error!("Failed to load users followed in channel: {}", e);
            Err(anyhow::anyhow!("Database query error: {}", e))
        }
    }
}

pub fn is_user_followed_in_channel(user_id: i32, channel: String) -> bool {
    let mut conn = match get_connection() {
        Some(c) => c,
        None => {
            tracing::error!("Failed to get DB connection");
            return false;
        }
    };

    match following::table
        .filter(following::followed_user_id.eq(user_id))
        .filter(following::channel_id.eq(channel))
        .first::<crate::models::Following>(&mut conn)
    {
        Ok(_) => true,
        Err(diesel::result::Error::NotFound) => false,
        Err(e) => {
            tracing::error!("Database query error: {}", e);
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
            tracing::error!("Failed to follow user: {}", e);
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
            .filter(following::channel_id.eq(channel)),
    )
    .get_result::<Following>(&mut conn)
    {
        Ok(deleted) => Ok(deleted),
        Err(e) => {
            tracing::error!("Failed to unfollow user: {}", e);
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
            tracing::error!("Failed to update webhook: {}", e);
            Err(anyhow::anyhow!("Database update error: {}", e))
        }
    }
}

fn get_connection() -> Option<DbConnection> {
    match crate::provider::db::DbProvider::global().get_connection() {
        Ok(c) => Some(c),
        Err(e) => {
            tracing::error!("Failed to get DB connection: {}", e);
            None
        }
    }
}
