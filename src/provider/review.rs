use crate::models::{NewReview, Review, ReviewWithUser, User};
use crate::provider::db::DbConnection;
use crate::provider::user;
use crate::provider::user::gmaps_user_id_to_db_id;
use crate::schema::reviews;
use crate::schema::users;
use diesel::prelude::*;

pub fn get_latest_review_for_user_gmaps_id(gmaps_id: &str) -> Option<ReviewWithUser> {
    get_latest_review_for_user(gmaps_user_id_to_db_id(gmaps_id)?)
}

pub fn get_new_review(user_id: i32) -> Option<ReviewWithUser> {
    let old_review_opt = match get_latest_review_from_db(user_id) {
        Some(r) => Some(r),
        None => return None,
    };
    let new_review = match old_review_opt {
        None => {
            let user = match user::get_user_from_id(user_id) {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("Failed to get user with id {}: {}", user_id, e);
                    return None;
                }
            };
            fetch_and_save_latest_review(&user)?
        }
        Some(ref r) => {
            let age_limit_hours = crate::config::get_config().review_age_limit_hours;
            let age_limit_duration = chrono::Duration::hours(age_limit_hours);
            let cutoff_time = (chrono::Utc::now() - age_limit_duration).naive_utc();
            if r.review.found_at <= cutoff_time {
                fetch_and_save_latest_review(&r.user)?
            } else {
                return None;
            }
        }
    };

    let old_review = old_review_opt.unwrap();
    if old_review.review.place_name != new_review.review.place_name
        && old_review.review.text != new_review.review.text
        && old_review.review.stars != new_review.review.stars
    {
        Some(new_review)
    } else {
        tracing::debug!("No new review found for user id '{}' latest known at '{}' found at '{}'", user_id, old_review.review.found_at, new_review.review.found_at);
        None
    }
}

pub fn get_latest_review_for_user(user_id: i32) -> Option<ReviewWithUser> {
    let latest = get_latest_review_from_db(user_id);
    if let Some(latest_review) = latest {
        let age_limit_hours = crate::config::get_config().review_age_limit_hours;
        let age_limit_duration = chrono::Duration::hours(age_limit_hours);
        let cutoff_time = (chrono::Utc::now() - age_limit_duration).naive_utc();
        if latest_review.review.found_at >= cutoff_time {
            Some(latest_review)
        } else {
            let user = match user::get_user_from_id(user_id) {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("Failed to get user with id {}: {}", user_id, e);
                    return None;
                }
            };
            fetch_and_save_latest_review(&user)
        }
    } else {
        let user = match user::get_user_from_id(user_id) {
            Ok(u) => u,
            Err(e) => {
                tracing::error!("Failed to get user with id {}: {}", user_id, e);
                return None;
            }
        };
        fetch_and_save_latest_review(&user)
    }
}

fn get_latest_review_from_db(user_id: i32) -> Option<ReviewWithUser> {
    let mut conn = get_connection()?;

    users::table
        .inner_join(reviews::table)
        .filter(users::id.eq(user_id))
        .order(reviews::found_at.desc())
        .select((User::as_select(), Review::as_select()))
        .first::<(User, Review)>(&mut conn)
        .map(|(user, review)| ReviewWithUser { user, review })
        .ok()
}

fn fetch_and_save_latest_review(user: &User) -> Option<ReviewWithUser> {
    let new_review = match crate::crawler::pages::review::get_latest_review_for_user(user) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to fetch latest review from Google Maps: {}", e);
            return None;
        }
    };

    save_new_review(&new_review)
}

fn save_new_review(new_review: &NewReview) -> Option<ReviewWithUser> {
    delete_reviews_for_user(new_review.user_id);

    let mut conn = get_connection()?;

    match diesel::insert_into(reviews::table)
        .values(new_review)
        .get_result::<Review>(&mut conn)
    {
        Ok(saved_review) => {
            let user = users::table
                .filter(users::id.eq(new_review.user_id))
                .first::<User>(&mut conn)
                .ok()?;

            Some(ReviewWithUser {
                review: saved_review,
                user,
            })
        }
        Err(e) => {
            tracing::error!("Failed to save new review to database: {}", e);
            None
        }
    }
}

fn delete_reviews_for_user(user_id: i32) -> Option<()> {
    let mut conn = get_connection()?;

    match diesel::delete(reviews::table.filter(reviews::user_id.eq(user_id))).execute(&mut conn) {
        Ok(_) => Some(()),
        Err(e) => {
            tracing::error!("Failed to delete old reviews for user {}: {}", user_id, e);
            None
        }
    }
}

fn get_connection() -> Option<DbConnection> {
    match crate::provider::db::DbProvider::global().get_connection() {
        Ok(c) => Some(c),
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            None
        }
    }
}
