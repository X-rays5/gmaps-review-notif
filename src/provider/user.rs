use crate::models::{NewUser, User};
use crate::provider::db::get_connection;
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

pub fn get_user_from_db_id(user_id: i32) -> Option<User> {
    let mut conn = get_connection()?;

    users::table
        .filter(users::id.eq(user_id))
        .first::<User>(&mut conn)
        .optional()
        .unwrap_or_else(|e| {
            tracing::error!(db_user_id = user_id, error = %e, "Failed to query user by db id");
            None
        })
}

pub fn gmaps_user_id_to_db_id(gmaps_id: &str) -> Option<i32> {
    match get_user_from_gmaps_id(gmaps_id) {
        Ok(u) => Some(u.id),
        Err(e) => {
            tracing::error!(gmaps_id = %gmaps_id, error = %e, "Failed to get user by gmaps_id");
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
            tracing::error!(gmaps_id = %gmaps_id, error = %e, "Failed to query user by gmaps_id");
            None
        })
}

fn fetch_and_save_user(gmaps_id: &str) -> Option<User> {
    let new_user = match crate::crawler::pages::user::get_user_from_id(gmaps_id) {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(gmaps_id = %gmaps_id, error = %e, "Failed to fetch user from Google Maps");
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
        Ok(saved_user) => {
            tracing::info!(db_user_id = saved_user.id, gmaps_id = %saved_user.gmaps_id, name = %saved_user.name, "Saved new user");
            Some(saved_user)
        }
        Err(e) => {
            tracing::error!(gmaps_id = %new_user.gmaps_id, error = %e, "Failed to save new user to database");
            None
        }
    }
}
