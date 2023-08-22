use std::future::{ready, Ready};

use crate::{
    models::{
        custom_api_errors::{ApiError, Result},
        session::{NewSession, Session},
        user::{NewUser, User},
        users_sessions::UsersSession,
    },
    schema::{sessions, users, users_sessions},
    utils::envs::{self},
};
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};

#[derive(Clone)]
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

    pub fn connection(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().expect("Could not get connection from pool")
    }

    pub fn create_session(
        &self,
        new_session: NewSession,
        creating_user_id: String,
    ) -> Result<Session> {
        let session = diesel::insert_into(sessions::table)
            .values(&new_session)
            .returning(Session::as_returning())
            .get_result(&mut self.connection())?;

        let new_users_sessions = UsersSession {
            user_id: creating_user_id,
            session_id: session.id.clone(),
        };

        diesel::insert_into(users_sessions::table)
            .values(&new_users_sessions)
            .execute(&mut self.connection())?;

        Ok(session)
    }

    pub fn get_sessions(&self) -> Result<Vec<Session>> {
        use crate::schema::sessions::dsl::sessions;

        let all_sessions = sessions
            .select(Session::as_select())
            .load(&mut self.connection())?;

        Ok(all_sessions)
    }

    pub fn get_session_by_id(&self, session_id: String) -> Result<Session> {
        use crate::schema::sessions::dsl::{id, sessions};

        let session = sessions
            .filter(id.eq(session_id))
            .select(Session::as_select())
            .first(&mut self.connection())?;

        Ok(session)
    }

    pub fn join_session(&self, session_id: String, user_id: String) -> Result<Session> {
        let new_users_sessions = UsersSession {
            user_id: user_id.clone(),
            session_id: session_id.clone(),
        };

        diesel::insert_into(users_sessions::table)
            .values(&new_users_sessions)
            .execute(&mut self.connection())?;

        let session = self.get_session_by_id(session_id)?;

        Ok(session)
    }

    pub fn create_user(&self, user: NewUser) -> Result<User> {
        let user = diesel::insert_into(users::table)
            .values(&user)
            .returning(User::as_returning())
            .get_result(&mut self.connection())?;

        Ok(user)
    }

    pub fn update_user(&self, user: NewUser) -> Result<User> {
        use crate::schema::users::dsl::{id, name};

        let user = diesel::update(users::table)
            .filter(id.eq(user.id))
            .set(name.eq(user.name))
            .returning(User::as_returning())
            .get_result(&mut self.connection())?;

        Ok(user)
    }

    pub fn get_user_by_id(&self, user_id: String) -> Result<User> {
        use crate::schema::users::dsl::{id, users};

        let user = users
            .filter(id.eq(user_id))
            .select(User::as_select())
            .first(&mut self.connection())?;

        Ok(user)
    }
}
