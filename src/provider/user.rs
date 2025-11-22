use diesel::prelude::*;
use anyhow::{anyhow, Result};
use crate::models::{NewUser, User};
use crate::provider::db::DbConnection;
use crate::schema::users;

pub fn get_user_from_gmaps_id(gmaps_id: String) -> Result<User> {
    match get_user_from_gmaps_id_db(&gmaps_id) {
        Some(u) => Ok(u),
        None => match fetch_and_save_user(&gmaps_id) {
            Some(new_user) => Ok(new_user),
            None => {
                Err(anyhow!("Failed to fetch user with gmaps_id: {}", gmaps_id))
            },
        },
    }
}

fn get_user_from_gmaps_id_db(gmaps_id: &str) -> Option<User> {
    let mut conn = get_connection()?;

    users::table
        .filter(users::gmaps_id.eq(gmaps_id.to_string()))
        .first::<User>(&mut conn)
        .optional().unwrap_or_else(|e| {
        tracing::error!("Database query error: {}", e);
        None
    })
}

fn fetch_and_save_user(gmaps_id: &str) -> Option<User> {
    let new_user = match crate::crawler::pages::user::get_user_from_id(gmaps_id) {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to fetch user from Google Maps: {}", e);
            return None;
        },
    };

    save_new_user(&new_user)
}

fn save_new_user(new_user: &NewUser) -> Option<User> {
    let mut conn = get_connection()?;

    match diesel::insert_into(users::table)
        .values(new_user)
        .get_result::<User>(&mut conn) {
        Ok(saved_user) => Some(saved_user),
        Err(e) => {
            tracing::error!("Failed to save new user to database: {}", e);
            None
        },
    }
}

fn get_connection() -> Option<DbConnection> {
    match crate::provider::db::DbProvider::global().get_connection() {
        Ok(c) => Some(c),
        Err(e) => {
            tracing::error!("Failed to get DB connection: {}", e);
            None
        },
    }
}