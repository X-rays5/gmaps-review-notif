use crate::models::{NewReview, Review, ReviewWithUser, User};
use crate::provider::db::DbConnection;
use crate::provider::user;
use crate::schema::reviews;
use crate::schema::users;
use diesel::prelude::*;

pub fn get_latest_review_for_user(gmaps_id: &str) -> Option<ReviewWithUser> {
    match get_latest_review_from_db(gmaps_id) {
        Some(r) => {
            if (r.review.found_at + chrono::Duration::hours(1)).and_utc() < chrono::Utc::now() {
                fetch_and_save_latest_review(gmaps_id)
            } else {
                Some(r)
            }
        }
        None => fetch_and_save_latest_review(gmaps_id),
    }
}

fn get_latest_review_from_db(gmaps_id: &str) -> Option<ReviewWithUser> {
    let mut conn = get_connection()?;

    users::table
        .inner_join(reviews::table)
        .filter(users::gmaps_id.eq(gmaps_id))
        .order(reviews::found_at.desc())
        .select((User::as_select(), Review::as_select()))
        .first::<(User, Review)>(&mut conn)
        .map(|(user, review)| ReviewWithUser { user, review })
        .ok()
}

fn fetch_and_save_latest_review(gmaps_id: &str) -> Option<ReviewWithUser> {
    let gmaps_user = match user::get_user_from_gmaps_id(gmaps_id.to_string()) {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to get user for gmaps_id {}: {}", gmaps_id, e);
            return None;
        }
    };

    let new_review = match crate::crawler::pages::review::get_latest_review_for_user(gmaps_user) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to fetch latest review from Google Maps: {}", e);
            return None;
        }
    };

    save_new_review(&new_review)
}

fn save_new_review(new_review: &NewReview) -> Option<ReviewWithUser> {
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

fn get_connection() -> Option<DbConnection> {
    match crate::provider::db::DbProvider::global().get_connection() {
        Ok(c) => Some(c),
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            None
        }
    }
}
