use crate::models::{NewReview, Review, ReviewWithUser, User};
use crate::provider::db::DbConnection;
use crate::provider::user::gmaps_user_id_to_db_id;
use crate::schema::reviews;
use crate::schema::users;
use diesel::prelude::*;

pub fn get_latest_review_for_user_gmaps_id(gmaps_id: &str) -> Option<ReviewWithUser> {
    get_latest_review_for_user(gmaps_user_id_to_db_id(gmaps_id)?)
}

pub fn check_for_new_review(user: &User) -> Option<ReviewWithUser> {
    let Some(old_review) = get_latest_review_from_db(user.id) else { return fetch_and_save_latest_review(user) };
    if !is_review_past_age_limit(&old_review.review) {
        return None;
    }

    let latest_review = fetch_latest_review(user)?;
    if is_new_review_different(&old_review.review, &latest_review) {
        save_new_review(&latest_review)
    } else {
        None
    }
}

pub fn get_latest_review_for_user(user_id: i32) -> Option<ReviewWithUser> {
    let old_review = get_latest_review_from_db(user_id)?;
    if !is_review_past_age_limit(&old_review.review) {
        return None;
    }

    let latest_review = fetch_latest_review(&old_review.user)?;
    if is_new_review_different(&old_review.review, &latest_review) {
        save_new_review(&latest_review)
    } else {
        Some(old_review)
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
    let new_review = fetch_latest_review(user)?;
    save_new_review(&new_review)
}

fn fetch_latest_review(user: &User) -> Option<NewReview> {
    match crate::crawler::pages::review::get_latest_review_for_user(user) {
        Ok(r) => Some(r),
        Err(e) => {
            tracing::error!("Failed to fetch latest review from Google Maps: {}", e);
            None
        }
    }
}

fn save_new_review(new_review: &NewReview) -> Option<ReviewWithUser> {
    let mut conn = get_connection()?;

    match conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::delete(reviews::table.filter(reviews::user_id.eq(new_review.user_id)))
            .execute(conn)?;

        let saved_review = diesel::insert_into(reviews::table)
            .values(new_review)
            .get_result::<Review>(conn)?;

        let user = users::table
            .filter(users::id.eq(new_review.user_id))
            .first::<User>(conn)?;

        Ok(ReviewWithUser {
            review: saved_review,
            user,
        })
    }) {
        Ok(result) => Some(result),
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

fn is_review_past_age_limit(review: &Review) -> bool {
    let age_limit_hours = crate::config::get_config().review_age_limit_hours;
    let age_limit_duration = chrono::Duration::hours(age_limit_hours);
    let cutoff_time = (chrono::Utc::now() - age_limit_duration).naive_utc();
    review.found_at < cutoff_time
}

fn is_new_review_different(current: &Review, new: &NewReview) -> bool {
    current.place_name != new.place_name || current.stars != new.stars || current.original_text != new.original_text
}
