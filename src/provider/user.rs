use crate::models::{NewUser, User};
use crate::provider::db::DbConnection;
use crate::schema::users;
use anyhow::{anyhow, Result};
use diesel::prelude::*;

pub fn get_user_from_gmaps_id(gmaps_id: &str) -> Result<User> {
    match get_user_from_gmaps_id_db(gmaps_id) {
        Some(u) => Ok(u),
        None => match fetch_and_save_user(gmaps_id) {
            Some(new_user) => Ok(new_user),
            None => Err(anyhow!("Failed to fetch user with gmaps_id: {gmaps_id}")),
        },
    }
}

pub fn get_user_from_id(user_id: i32) -> Result<User> {
    let mut conn = get_connection().ok_or_else(|| anyhow!("Failed to get DB connection"))?;

    match users::table
        .filter(users::id.eq(user_id))
        .first::<User>(&mut conn)
    {
        Ok(user) => Ok(user),
        Err(e) => {
            tracing::error!("Database query error: {}", e);
            Err(anyhow!("Failed to find user with id: {user_id}"))
        }
    }
}

pub fn gmaps_user_id_to_db_id(gmaps_id: &str) -> Option<i32> {
    match get_user_from_gmaps_id(gmaps_id) {
        Ok(u) => Some(u.id),
        Err(e) => {
            tracing::error!("Failed to get user from gmaps_id {}: {}", gmaps_id, e);
            None
        }
    }
}

fn get_user_from_gmaps_id_db(gmaps_id: &str) -> Option<User> {
    let mut conn = get_connection()?;

    users::table
        .filter(users::gmaps_id.eq(gmaps_id.to_string()))
        .first::<User>(&mut conn)
        .optional()
        .unwrap_or_else(|e| {
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
        }
    };

    save_new_user(&new_user)
}

fn save_new_user(new_user: &NewUser) -> Option<User> {
    let mut conn = get_connection()?;

    match diesel::insert_into(users::table)
        .values(new_user)
        .get_result::<User>(&mut conn)
    {
        Ok(saved_user) => Some(saved_user),
        Err(e) => {
            tracing::error!("Failed to save new user to database: {}", e);
            None
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
