use crate::models::{NewReview, Review, ReviewWithUser, User};
use crate::provider::db::DbConnection;
use crate::provider::user::{get_user_from_db_id, gmaps_user_id_to_db_id};
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
    let latest_in_db = get_latest_review_from_db(user_id);
    if let Some(latest) = latest_in_db.as_ref() {
        if !is_review_past_age_limit(&latest.review) {
            return latest_in_db;
        }
    }

    let Some(user) = get_user_from_db_id(user_id) else {
        tracing::error!("Failed to get user from db: {}", user_id);
        return None;
    };

    match check_for_new_review(&user) {
        Some(new_user) => Some(new_user),
        None => latest_in_db
    }
}

fn get_latest_review_from_db(user_id: i32) -> Option<ReviewWithUser> {
    let mut conn = get_connection()?;

    users::table
        .inner_join(reviews::table)
        .filter(users::id.eq(user_id))
        .order(reviews::found_at.desc())
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

        diesel::insert_into(reviews::table)
            .values(new_review)
            .execute(conn)?;

        let saved_review = reviews::table
            .filter(reviews::user_id.eq(new_review.user_id))
            .order(reviews::found_at.desc())
            .first::<Review>(conn)?;

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
    let place_name_changed = current.place_name != new.place_name;
    let stars_changed = current.stars != new.stars;
    let original_text_changed = current.original_text != new.original_text;

    // Compare pictures by count only because URLs are not stable.
    let current_pic_count = extract_picture_count(&current.pictures);
    let new_pic_count = extract_picture_count(&new.pictures);
    let pictures_changed = current_pic_count != new_pic_count;

    let is_different = place_name_changed || stars_changed || original_text_changed || pictures_changed;
    if !is_different {
        return false;
    }

    if tracing::enabled!(tracing::Level::INFO) {
        let mut changed_fields = Vec::new();
        if place_name_changed {
            changed_fields.push("place_name");
        }
        if stars_changed {
            changed_fields.push("stars");
        }
        if original_text_changed {
            changed_fields.push("original_text");
        }
        if pictures_changed {
            changed_fields.push("pictures");
        }

        let mut change_details = Vec::new();
        if place_name_changed {
            change_details.push(format!(
                "place_name: {:?} -> {:?}",
                current.place_name, new.place_name
            ));
        }
        if stars_changed {
            change_details.push(format!("stars: {} -> {}", current.stars, new.stars));
        }
        if original_text_changed {
            change_details.push(format!(
                "original_text: {:?} -> {:?}",
                current.original_text, new.original_text
            ));
        }
        if pictures_changed {
            change_details.push(format!(
                "picture_count: {} -> {}",
                current_pic_count, new_pic_count
            ));
        }

        tracing::info!(
            user_id = new.user_id,
            changed_fields = ?changed_fields,
            changes = ?change_details,
            "Detected new review differences"
        );
    }
    true
}

fn extract_picture_count(pictures: &serde_json::Value) -> usize {
    pictures
        .as_array()
        .map(|arr| arr.iter().filter(|v| v.as_str().is_some()).count())
        .unwrap_or_default()
}
