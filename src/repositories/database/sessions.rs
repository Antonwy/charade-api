use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::models::custom_api_errors::Result;
use crate::models::session::{NewSession, Session, SessionInfo};
use crate::models::users_sessions::UsersSession;
use crate::schema;

use super::Database;

impl Database {
    pub async fn create_session(&self, new_session: NewSession) -> Result<Session> {
        use schema::{sessions, users_sessions};

        let mut connection = self.connection().await?;

        let session = diesel::insert_into(sessions::table)
            .values(&new_session)
            .returning(Session::as_returning())
            .get_result(&mut connection)
            .await?;

        let new_users_sessions = UsersSession {
            user_id: new_session.admin_user_id.to_string(),
            session_id: session.id.clone(),
        };

        diesel::insert_into(users_sessions::table)
            .values(&new_users_sessions)
            .execute(&mut connection)
            .await?;

        Ok(session)
    }

    pub async fn get_sessions(&self) -> Result<Vec<Session>> {
        use schema::sessions::dsl::sessions;

        let all_sessions = sessions
            .select(Session::as_select())
            .load(&mut self.connection().await?)
            .await?;

        Ok(all_sessions)
    }

    pub async fn get_session_by_id(&self, session_id: &str) -> Result<Session> {
        use crate::schema::sessions::dsl::{id, sessions};

        let session = sessions
            .filter(id.eq(session_id))
            .select(Session::as_select())
            .first(&mut self.connection().await?)
            .await?;

        Ok(session)
    }

    pub async fn join_session(&self, session_id: &str, user_id: &str) -> Result<Session> {
        use crate::schema::users_sessions;

        let new_users_sessions = UsersSession {
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
        };

        diesel::insert_into(users_sessions::table)
            .values(&new_users_sessions)
            .execute(&mut self.connection().await?)
            .await?;

        let session = self.get_session_by_id(session_id).await?;

        Ok(session)
    }

    pub async fn get_session_info(&self, session_id: &str) -> Result<SessionInfo> {
        let session = self.get_session_by_id(session_id).await?;

        let words = self.get_words_by_session_id(session_id).await?;

        let users = self.get_users_by_session_id(session_id).await?;

        Ok(SessionInfo {
            session,
            words,
            users,
        })
    }

    pub async fn get_sessions_by_user(&self, user_id: &str) -> Result<Vec<Session>> {
        use schema::sessions::table as sessions_table;
        use schema::users_sessions::{dsl::users_sessions, user_id as user_id_column};

        let sessions = users_sessions
            .inner_join(sessions_table)
            .filter(user_id_column.eq(user_id))
            .select(Session::as_select())
            .get_results::<Session>(&mut self.connection().await?)
            .await?;

        Ok(sessions)
    }
}
