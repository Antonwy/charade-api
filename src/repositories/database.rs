use crate::{
    models::custom_api_errors::{ApiError, Result},
    utils::envs,
};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

mod sessions;
mod users;
mod words;

type AsyncDbConnectionPool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
type PooledAsyncDbConnection<'a> =
    bb8::PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(Clone, Debug)]
pub struct Database {
    pool: AsyncDbConnectionPool,
}

impl Database {
    pub async fn init() -> Self {
        let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
            envs::database_url(),
        );

        let pool = bb8::Pool::builder()
            .build(config)
            .await
            .expect("Could not build bb8 pool");

        Self { pool }
    }

    pub async fn connection(&self) -> Result<PooledAsyncDbConnection, ApiError> {
        self.pool
            .get()
            .await
            .map_err(|_| ApiError::internal("Could not get database connection".to_string()))
    }
}
