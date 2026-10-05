use crate::config::get_config;
use anyhow::{Error, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use std::sync::OnceLock;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

static DB_INSTANCE: OnceLock<DbProvider> = OnceLock::new();

pub struct DbProvider {
    pool: Option<DbPool>,
}

impl DbProvider {
    fn new() -> Self {
        let manager = ConnectionManager::<PgConnection>::new(get_config().database_url.clone());
        match r2d2::Pool::builder().build(manager) {
            Ok(pool) => DbProvider { pool: Some(pool) },
            Err(e) => {
                tracing::error!(error = %e, "Failed to create the database connection pool");
                DbProvider { pool: None }
            }
        }
    }

    pub fn global() -> &'static DbProvider {
        DB_INSTANCE.get_or_init(DbProvider::new)
    }

    pub fn get_connection(&self) -> Result<DbConnection> {
        let Some(pool) = self.pool.as_ref() else {
            return Err(Error::msg("The database connection pool is unavailable"));
        };

        pool.get().map_err(Error::from)
    }
}

/// Shared helper for providers: fetches a pooled connection, logging the failure once with context.
pub fn get_connection() -> Option<DbConnection> {
    match DbProvider::global().get_connection() {
        Ok(conn) => Some(conn),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get a database connection from the pool");
            None
        }
    }
}
