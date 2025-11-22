use crate::config::get_config;
use anyhow::{Error, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use std::sync::OnceLock;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

static DB_INSTANCE: OnceLock<DbProvider> = OnceLock::new();

pub struct DbProvider {
    pub pool: DbPool,
}

impl DbProvider {
    fn new() -> Self {
        let manager = ConnectionManager::<PgConnection>::new(get_config().database_url.clone());
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create DB pool");

        DbProvider { pool }
    }

    pub fn global() -> &'static DbProvider {
        DB_INSTANCE.get_or_init(|| DbProvider::new())
    }

    pub fn get_connection(&self) -> Result<DbConnection> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(Error::from(e)),
        }
    }
}
