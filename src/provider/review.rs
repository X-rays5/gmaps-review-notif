use crate::models::{NewReview, NewSeenReview, Review, ReviewWithUser, User};
use crate::provider::db::get_connection;
use crate::provider::user::{get_user_from_db_id, gmaps_user_id_to_db_id};
use crate::schema::reviews;
use crate::schema::seen_reviews;
use crate::schema::users;
use crate::utility::shorten::shorten_url;
use diesel::prelude::*;
use reqwest::Url;
use sha2::{Digest, Sha256};

pub fn get_latest_review_for_user_gmaps_id(gmaps_id: &str) -> Option<ReviewWithUser> {
    get_latest_review_for_user(gmaps_user_id_to_db_id(gmaps_id)?)
}

pub fn check_for_new_review(user: &User) -> Option<ReviewWithUser> {
    let Some(old_review) = get_latest_review_from_db(user.id) else {
        tracing::debug!(db_user_id = user.id, gmaps_id = %user.gmaps_id, "No stored review yet; fetching first review");
        return fetch_and_save_latest_review(user);
    };
    if !is_review_past_age_limit(&old_review.review) {
        tracing::debug!(
            db_user_id = user.id,
            gmaps_id = %user.gmaps_id,
            found_at = %old_review.review.found_at,
            "Stored review is still fresh; skipping check"
        );
        return None;
    }

    // Make sure the review we currently hold is remembered, so a crawl that returns it
    // (or any review we have already seen) is not mistaken for a new review.
    remember_review(&old_review.review);

    let latest_review = fetch_latest_review(user)?;

    // The crawler sometimes falls back to an older review (e.g. a page still loading). If we
    // have seen this review before it is not new, so never report it as one.
    if is_review_already_seen(user.id, &latest_review) {
        tracing::info!(
            db_user_id = user.id,
            gmaps_id = %user.gmaps_id,
            place_name = %latest_review.place_name,
            "Crawled review has already been seen before, likely an older review; skipping"
        );
        return None;
    }

    if is_new_review_different(&old_review.review, &latest_review) {
        tracing::info!(
            db_user_id = user.id,
            gmaps_id = %user.gmaps_id,
            place_name = %latest_review.place_name,
            stars = latest_review.stars,
            "New review detected"
        );
        save_new_review(&latest_review)
    } else {
        tracing::debug!(
            db_user_id = user.id,
            gmaps_id = %user.gmaps_id,
            "Crawled review is unchanged from the stored one; skipping"
        );
        None
    }
}

pub fn get_latest_review_for_user(user_id: i32) -> Option<ReviewWithUser> {
    let latest_in_db = get_latest_review_from_db(user_id);
    if let Some(latest) = latest_in_db.as_ref()
        && !is_review_past_age_limit(&latest.review)
    {
        tracing::debug!(db_user_id = user_id, review_id = latest.review.id, "Returning cached review (still fresh)");
        return latest_in_db;
    }

    let Some(user) = get_user_from_db_id(user_id) else {
        tracing::error!(db_user_id = user_id, "Failed to get user from db");
        return None;
    };

    match check_for_new_review(&user) {
        Some(new_user) => Some(new_user),
        None => latest_in_db
    }
}

fn get_latest_review_from_db(user_id: i32) -> Option<ReviewWithUser> {
    let mut conn = get_connection()?;

    match users::table
        .inner_join(reviews::table)
        .filter(users::id.eq(user_id))
        .order(reviews::found_at.desc())
        .first::<(User, Review)>(&mut conn)
    {
        Ok((user, review)) => Some(ReviewWithUser { user, review }),
        Err(diesel::result::Error::NotFound) => None,
        Err(e) => {
            tracing::error!(db_user_id = user_id, error = %e, "Failed to load the latest review from the database");
            None
        }
    }
}

fn fetch_and_save_latest_review(user: &User) -> Option<ReviewWithUser> {
    let new_review = fetch_latest_review(user)?;
    save_new_review(&new_review)
}

fn fetch_latest_review(user: &User) -> Option<NewReview> {
    match crate::crawler::pages::review::get_latest_review_for_user(user) {
        Ok(r) => Some(r),
        Err(e) => {
            tracing::error!(db_user_id = user.id, gmaps_id = %user.gmaps_id, error = %e, "Failed to fetch latest review from Google Maps");
            None
        }
    }
}

fn save_new_review(new_review: &NewReview) -> Option<ReviewWithUser> {
    let mut conn = get_connection()?;

    // Shorten the review URL
    let shortened_url = match Url::parse(&new_review.link_en) {
        Ok(url) => match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                shorten_url(&url).await
            })
        }) {
            Ok(shortened) => shortened,
            Err(e) => {
                tracing::warn!(db_user_id = new_review.user_id, error = %e, "Failed to shorten review URL, using original URL");
                new_review.link_en.clone()
            }
        },
        Err(e) => {
            tracing::warn!(db_user_id = new_review.user_id, error = %e, "Failed to parse review URL, using original URL");
            new_review.link_en.clone()
        }
    };

    // Shorten picture URLs
    let shortened_pictures = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            shorten_picture_urls_async(&new_review.pictures).await
        })
    });

    // Create a modified review with shortened URLs
    let mut modified_review = new_review.clone();
    modified_review.link_en = shortened_url;
    modified_review.pictures = shortened_pictures;

    match conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::delete(reviews::table.filter(reviews::user_id.eq(modified_review.user_id)))
            .execute(conn)?;

        diesel::insert_into(reviews::table)
            .values(&modified_review)
            .execute(conn)?;

        diesel::insert_into(seen_reviews::table)
            .values(&NewSeenReview {
                user_id: modified_review.user_id,
                hash: new_review_hash(&modified_review),
            })
            .on_conflict_do_nothing()
            .execute(conn)?;

        let saved_review = reviews::table
            .filter(reviews::user_id.eq(modified_review.user_id))
            .order(reviews::found_at.desc())
            .first::<Review>(conn)?;

        let user = users::table
            .filter(users::id.eq(modified_review.user_id))
            .first::<User>(conn)?;

        Ok(ReviewWithUser {
            review: saved_review,
            user,
        })
    }) {
        Ok(result) => {
            tracing::info!(
                db_user_id = result.review.user_id,
                review_id = result.review.id,
                place_name = %result.review.place_name,
                "Saved new review"
            );
            Some(result)
        }
        Err(e) => {
            tracing::error!(db_user_id = modified_review.user_id, error = %e, "Failed to save new review to database");
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
    let original_text_changed = if new.original_text.is_some() { current.original_text != new.original_text } else { false };

    // Compare pictures by count only because URLs are not stable.
    let current_pic_count = extract_picture_count(&current.pictures);
    let new_pic_count = extract_picture_count(&new.pictures);
    let pictures_changed = current_pic_count != new_pic_count;

    let is_different = place_name_changed || stars_changed || original_text_changed || pictures_changed;
    if !is_different {
        return false;
    }

    if tracing::enabled!(tracing::Level::DEBUG) {
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
                "picture_count: {current_pic_count} -> {new_pic_count}"
            ));
        }

        tracing::debug!(
            db_user_id = new.user_id,
            changed_fields = ?changed_fields,
            changes = ?change_details,
            "Detected review field differences"
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

/// The review's stable textual content: the original text when Google shows a translation,
/// otherwise the displayed text. Comparing this stays consistent whether or not a translation
/// happens to be available on a given crawl.
fn review_content_key(review: &Review) -> &str {
    review.original_text.as_deref().unwrap_or(review.text.as_str())
}

fn new_review_content_key(review: &NewReview) -> &str {
    review.original_text.as_deref().unwrap_or(review.text.as_str())
}

/// SHA-256 over the review's title (place name) and body (text). Stars and pictures are
/// deliberately excluded: stars are not part of a review's identity here, and picture count can
/// vary between crawls (lazy loading) — either would produce false "new" hashes.
fn compute_review_hash(place_name: &str, content: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(place_name.as_bytes());
    hasher.update([0u8]);
    hasher.update(content.as_bytes());
    hasher.finalize().to_vec()
}

fn review_hash(review: &Review) -> Vec<u8> {
    compute_review_hash(&review.place_name, review_content_key(review))
}

fn new_review_hash(review: &NewReview) -> Vec<u8> {
    compute_review_hash(&review.place_name, new_review_content_key(review))
}

/// Records a review's hash so we can recognize it (and never report it as new) in the future.
fn remember_review(review: &Review) {
    let Some(mut conn) = get_connection() else { return };
    let seen_review = NewSeenReview {
        user_id: review.user_id,
        hash: review_hash(review),
    };
    if let Err(e) = diesel::insert_into(seen_reviews::table)
        .values(&seen_review)
        .on_conflict_do_nothing()
        .execute(&mut conn)
    {
        tracing::error!(db_user_id = review.user_id, error = %e, "Failed to record seen review hash");
    }
}

fn is_review_already_seen(user_id: i32, review: &NewReview) -> bool {
    let Some(mut conn) = get_connection() else { return false };
    seen_reviews::table
        .filter(seen_reviews::user_id.eq(user_id))
        .filter(seen_reviews::hash.eq(new_review_hash(review)))
        .select(seen_reviews::id)
        .first::<i32>(&mut conn)
        .optional()
        .map(|found| found.is_some())
        .unwrap_or_else(|e| {
            tracing::error!(db_user_id = user_id, error = %e, "Failed to check seen review hash");
            false
        })
}

async fn shorten_picture_urls_async(pictures: &serde_json::Value) -> serde_json::Value {
    match pictures.as_array() {
        Some(arr) => {
            let mut shortened_urls = Vec::new();
            for v in arr {
                if let Some(url_str) = v.as_str() {
                    match Url::parse(url_str) {
                        Ok(url) => match shorten_url(&url).await {
                            Ok(shortened) => {
                                shortened_urls.push(serde_json::Value::String(shortened));
                            }
                            Err(e) => {
                                tracing::warn!(error = %e, "Failed to shorten picture URL, using original URL");
                                shortened_urls.push(serde_json::Value::String(url_str.to_string()));
                            }
                        },
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to parse picture URL, using original URL");
                            shortened_urls.push(serde_json::Value::String(url_str.to_string()));
                        }
                    }
                } else {
                    shortened_urls.push(v.clone());
                }
            }
            serde_json::Value::Array(shortened_urls)
        }
        None => pictures.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compute_review_hash, extract_picture_count, is_new_review_different, new_review_hash,
        review_hash, shorten_picture_urls_async,
    };
    use crate::models::{NewReview, Review};
    use chrono::Utc;
    use serde_json::json;

    fn review_with(pictures: serde_json::Value, stars: i32, original_text: Option<&str>) -> Review {
        Review {
            id: 1,
            place_name: "Place".to_string(),
            text: "Text".to_string(),
            original_text: original_text.map(str::to_string),
            stars,
            user_id: 42,
            found_at: Utc::now().naive_utc(),
            link_en: Some("https://example.com".to_string()),
            pictures,
        }
    }

    fn new_review_with(
        pictures: serde_json::Value,
        stars: i32,
        original_text: Option<&str>,
    ) -> NewReview {
        NewReview {
            place_name: "Place".to_string(),
            text: "Text".to_string(),
            original_text: original_text.map(str::to_string),
            stars,
            user_id: 42,
            link_en: "https://example.com/new".to_string(),
            pictures,
        }
    }

    #[test]
    fn extract_picture_count_counts_only_string_urls() {
        let pictures = json!(["a", 1, null, "b", { "x": true }]);
        assert_eq!(extract_picture_count(&pictures), 2);
    }

    #[test]
    fn compute_review_hash_is_deterministic_and_sha256_sized() {
        let a = compute_review_hash("Place", "text");
        let b = compute_review_hash("Place", "text");
        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
    }

    #[test]
    fn compute_review_hash_differs_for_different_title_or_body() {
        let base = compute_review_hash("Place", "text");
        assert_ne!(base, compute_review_hash("Other Place", "text"));
        assert_ne!(base, compute_review_hash("Place", "different"));
    }

    #[test]
    fn review_hash_ignores_stars() {
        let stored = review_with(json!([]), 5, Some("hola"));
        let fetched = new_review_with(json!([]), 1, Some("hola"));

        assert_eq!(review_hash(&stored), new_review_hash(&fetched));
    }

    #[test]
    fn review_hash_ignores_whether_translation_was_shown() {
        // Same review, once stored with an original text (translation shown)...
        let stored = review_with(json!([]), 5, Some("hola"));
        // ...and once crawled without one (no translation offered, text is the original).
        let mut fetched = new_review_with(json!([]), 5, None);
        fetched.text = "hola".to_string();

        assert_eq!(review_hash(&stored), new_review_hash(&fetched));
    }

    #[test]
    fn review_hash_differs_for_different_text() {
        let stored = review_with(json!([]), 5, None);
        let mut fetched = new_review_with(json!([]), 5, None);
        fetched.text = "a different review".to_string();

        assert_ne!(review_hash(&stored), new_review_hash(&fetched));
    }

    #[test]
    fn extract_picture_count_returns_zero_for_non_array_values() {
        assert_eq!(extract_picture_count(&json!(null)), 0);
        assert_eq!(extract_picture_count(&json!({ "pictures": [] })), 0);
    }

    #[test]
    fn is_new_review_different_ignores_picture_url_changes_if_count_matches() {
        let current = review_with(json!(["https://old/1", "https://old/2"]), 5, Some("hola"));
        let new = new_review_with(json!(["https://new/a", "https://new/b"]), 5, Some("hola"));

        assert!(!is_new_review_different(&current, &new));
    }

    #[test]
    fn is_new_review_different_detects_star_change() {
        let current = review_with(json!(["https://img/1"]), 4, Some("same"));
        let new = new_review_with(json!(["https://img/2"]), 5, Some("same"));

        assert!(is_new_review_different(&current, &new));
    }

    #[test]
    fn is_new_review_different_detects_place_name_change() {
        let current = review_with(json!(["https://img/1"]), 5, Some("same"));
        let mut new = new_review_with(json!(["https://img/2"]), 5, Some("same"));
        new.place_name = "Another Place".to_string();

        assert!(is_new_review_different(&current, &new));
    }

    #[test]
    fn is_new_review_different_detects_original_text_change() {
        let current = review_with(json!(["https://img/1"]), 5, None);
        let new = new_review_with(json!(["https://img/2"]), 5, Some("original text"));

        assert!(is_new_review_different(&current, &new));
    }

    #[test]
    fn is_new_review_different_detects_picture_count_change() {
        let current = review_with(json!(["https://img/1"]), 5, Some("same"));
        let new = new_review_with(json!(["https://img/a", "https://img/b"]), 5, Some("same"));

        assert!(is_new_review_different(&current, &new));
    }

    #[test]
    fn is_new_review_different_ignores_translated_text_change_only() {
        let current = review_with(json!(["https://img/1"]), 5, Some("same"));
        let mut new = new_review_with(json!(["https://img/a"]), 5, Some("same"));
        new.text = "Different translated text".to_string();

        assert!(!is_new_review_different(&current, &new));
    }

    #[tokio::test]
    async fn shorten_picture_urls_preserves_non_string_elements() {
        let pictures = json!(["not-a-valid-url", 42, null, { "x": true }, "also-not-valid"]);
        let result = shorten_picture_urls_async(&pictures).await;
        let arr = result.as_array().expect("result should be an array");
        assert_eq!(arr.len(), 5);
        // Non-string values should be preserved as-is
        assert_eq!(arr[1], json!(42));
        assert_eq!(arr[2], json!(null));
        assert_eq!(arr[3], json!({ "x": true }));
        // Invalid URL strings fall back to the original string value
        assert_eq!(arr[0], json!("not-a-valid-url"));
        assert_eq!(arr[4], json!("also-not-valid"));
    }

    #[tokio::test]
    async fn shorten_picture_urls_returns_clone_for_non_array() {
        let pictures = json!({ "url": "https://example.com" });
        let result = shorten_picture_urls_async(&pictures).await;
        assert_eq!(result, pictures);
    }

    #[tokio::test]
    async fn shorten_picture_urls_returns_empty_array_unchanged() {
        let pictures = json!([]);
        let result = shorten_picture_urls_async(&pictures).await;
        assert_eq!(result, pictures);
    }
}

