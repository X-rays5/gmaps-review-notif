use crate::models::{NewReview, Review, ReviewWithUser, User};
use crate::provider::db::DbConnection;
use crate::provider::user;
use crate::schema::reviews;
use crate::schema::users;
use diesel::prelude::*;
use crate::provider::user::gmaps_user_id_to_db_id;

pub fn get_latest_review_for_user_gmaps_id(gmaps_id: &str) -> Option<ReviewWithUser> {
    get_latest_review_for_user(gmaps_user_id_to_db_id(gmaps_id)?)
}

pub fn get_latest_review_for_user(user_id: i32) -> Option<ReviewWithUser> {
    match get_latest_review_from_db(user_id) {
        Some(review) => Some(review),
        None => {
            let user = match user::get_user_from_id(user_id) {
                Ok(u) => u,
                Err(e) => {
                    tracing::error!("Failed to get user with id {}: {}", user_id, e);
                    return None;
                }
            };
            fetch_and_save_latest_review(user)
        }
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

fn fetch_and_save_latest_review(user: User) -> Option<ReviewWithUser> {
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
