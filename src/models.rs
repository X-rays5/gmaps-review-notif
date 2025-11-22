use crate::schema::{following, reviews, users};
use chrono::NaiveDateTime;
use diesel::prelude::*;

// --- USER MODELS ---
#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub gmaps_id: String,
    pub name: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser {
    pub gmaps_id: String,
    pub name: String,
}

// --- REVIEW MODELS ---
#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(table_name = reviews)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Review {
    pub id: i32,
    pub place_name: String,
    pub review_text: String,
    pub review_original_text: Option<String>,
    pub stars: i32,
    pub user_id: i32,
    pub found_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = reviews)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewReview {
    pub place_name: String,
    pub review_text: String,
    pub review_original_text: Option<String>,
    pub stars: i32,
    pub user_id: i32,
}

#[derive(Queryable, Debug)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ReviewWithUser {
    #[diesel(embed)]
    pub user: User,
    #[diesel(embed)]
    pub review: Review,
}

// --- FOLLOWING MODELS ---
#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(belongs_to(User, foreign_key = followed_user_id))]
#[diesel(table_name = following)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Following {
    pub id: i32,
    pub followed_user_id: i32,
    pub channel_id: String,
    pub original_text: bool,
    pub webhook_id: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = following)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFollowing {
    pub followed_user_id: i32,
    pub channel_id: String,
    pub original_text: bool,
    pub webhook_id: String,
}