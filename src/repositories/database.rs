use crate::{
    models::custom_api_errors::{ApiError, Result},
    utils::envs,
};
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};

mod sessions;
mod users;
mod words;

#[derive(Clone, Debug)]
pub struct Database {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Database {
    pub fn init() -> Self {
        let manager = ConnectionManager::<PgConnection>::new(envs::database_url());

        let connection_pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Could not build connection pool");

        Self {
            pool: connection_pool,
        }
    }

    pub fn connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, ApiError> {
        self.pool
            .get()
            .map_err(|_| ApiError::internal("Could not get database connection".to_string()))
    }
}
