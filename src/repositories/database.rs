use crate::{
    models::{
        custom_api_errors::{ApiError, Result},
        session::{NewSession, Session, SessionInfo},
        user::{FullUserInfo, NewUser, User},
        users_sessions::UsersSession,
        word::{NewWord, Word},
    },
    schema::{sessions, users, users_sessions, words},
    utils::envs::{self},
};
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
};

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

    pub fn create_session(&self, new_session: NewSession) -> Result<Session> {
        let mut connection = self.connection()?;

        let session = diesel::insert_into(sessions::table)
            .values(&new_session)
            .returning(Session::as_returning())
            .get_result(&mut connection)?;

        let new_users_sessions = UsersSession {
            user_id: new_session.admin_user_id.to_string(),
            session_id: session.id.clone(),
        };

        diesel::insert_into(users_sessions::table)
            .values(&new_users_sessions)
            .execute(&mut connection)?;

        Ok(session)
    }

    pub fn get_sessions(&self) -> Result<Vec<Session>> {
        use crate::schema::sessions::dsl::sessions;

        let all_sessions = sessions
            .select(Session::as_select())
            .load(&mut self.connection()?)?;

        Ok(all_sessions)
    }

    pub fn get_session_by_id(&self, session_id: &str) -> Result<Session> {
        use crate::schema::sessions::dsl::{id, sessions};

        let session = sessions
            .filter(id.eq(session_id))
            .select(Session::as_select())
            .first(&mut self.connection()?)?;

        Ok(session)
    }

    pub fn join_session(&self, session_id: &str, user_id: &str) -> Result<Session> {
        let new_users_sessions = UsersSession {
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
        };

        diesel::insert_into(users_sessions::table)
            .values(&new_users_sessions)
            .execute(&mut self.connection()?)?;

        let session = self.get_session_by_id(session_id)?;

        Ok(session)
    }

    pub fn get_session_info(&self, session_id: &str) -> Result<SessionInfo> {
        let session = self.get_session_by_id(session_id)?;

        let words = self.get_words_by_session_id(session_id)?;

        let users = self.get_users_by_session_id(session_id)?;

        Ok(SessionInfo {
            session,
            words,
            users,
        })
    }

    pub fn create_user(&self, user: NewUser) -> Result<User> {
        let user = diesel::insert_into(users::table)
            .values(&user)
            .returning(User::as_returning())
            .get_result(&mut self.connection()?)?;

        Ok(user)
    }

    pub fn update_user(&self, user: NewUser) -> Result<User> {
        use crate::schema::users::dsl::{id, name};

        let user = diesel::update(users::table)
            .filter(id.eq(user.id))
            .set(name.eq(user.name))
            .returning(User::as_returning())
            .get_result(&mut self.connection()?)?;

        Ok(user)
    }

    pub fn get_user_by_id(&self, user_id: &str) -> Result<User> {
        use crate::schema::users::dsl::{id, users};

        let user = users
            .filter(id.eq(user_id))
            .select(User::as_select())
            .first(&mut self.connection()?)?;

        Ok(user)
    }

    pub fn get_users_by_session_id(&self, session_id: &str) -> Result<Vec<User>> {
        use crate::schema::users::table as users_table;
        use crate::schema::users_sessions::{dsl::users_sessions, session_id as session_id_column};

        let users = users_sessions
            .inner_join(users_table)
            .filter(session_id_column.eq(session_id))
            .select(User::as_select())
            .get_results::<User>(&mut self.connection()?)?;

        Ok(users)
    }

    pub fn get_sessions_by_user(&self, user_id: &str) -> Result<Vec<Session>> {
        use crate::schema::sessions::table as sessions_table;
        use crate::schema::users_sessions::{dsl::users_sessions, user_id as user_id_column};

        let sessions = users_sessions
            .inner_join(sessions_table)
            .filter(user_id_column.eq(user_id))
            .select(Session::as_select())
            .get_results::<Session>(&mut self.connection()?)?;

        Ok(sessions)
    }

    pub fn get_full_user_info(&self, user_id: &str) -> Result<FullUserInfo> {
        let user = self.get_user_by_id(user_id)?;

        let sessions = self.get_sessions_by_user(user_id)?;

        Ok(FullUserInfo {
            user: user.clone(),
            sessions,
        })
    }

    pub fn add_word_to_session(&self, new_word: NewWord) -> Result<Word> {
        let word = diesel::insert_into(words::table)
            .values(&new_word)
            .returning(Word::as_returning())
            .get_result(&mut self.connection()?)?;

        Ok(word)
    }

    pub fn get_number_of_words_in_session(&self, session_id: &str) -> Result<u16> {
        use crate::schema::words::dsl::{session_id as session_id_column, words};

        let number_of_words: i64 = words
            .filter(session_id_column.eq(session_id))
            .count()
            .get_result(&mut self.connection()?)?;

        Ok(number_of_words as u16)
    }

    pub fn get_words_by_session_id(&self, session_id: &str) -> Result<Vec<Word>> {
        use crate::schema::words::dsl::{session_id as session_id_column, words};

        let w = words
            .filter(session_id_column.eq(session_id))
            .select(Word::as_select())
            .get_results(&mut self.connection()?)?;

        Ok(w)
    }
}
