use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::models::custom_api_errors::Result;
use crate::models::user::{FullUserInfo, NewUser, User};
use crate::schema;

use super::Database;

impl Database {
    pub fn create_user(&self, user: NewUser) -> Result<User> {
        let user = diesel::insert_into(schema::users::table)
            .values(&user)
            .returning(User::as_returning())
            .get_result(&mut self.connection()?)?;

        Ok(user)
    }

    pub fn update_user(&self, user: NewUser) -> Result<User> {
        use schema::users::dsl::{id, name};

        let user = diesel::update(schema::users::table)
            .filter(id.eq(user.id))
            .set(name.eq(user.name))
            .returning(User::as_returning())
            .get_result(&mut self.connection()?)?;

        Ok(user)
    }

    pub fn get_user_by_id(&self, user_id: &str) -> Result<User> {
        use schema::users::dsl::{id, users};

        let user = users
            .filter(id.eq(user_id))
            .select(User::as_select())
            .first(&mut self.connection()?)?;

        Ok(user)
    }

    pub fn get_users_by_session_id(&self, session_id: &str) -> Result<Vec<User>> {
        use schema::users::table as users_table;
        use schema::users_sessions::{dsl::users_sessions, session_id as session_id_column};

        let users = users_sessions
            .inner_join(users_table)
            .filter(session_id_column.eq(session_id))
            .select(User::as_select())
            .get_results::<User>(&mut self.connection()?)?;

        Ok(users)
    }

    pub fn get_full_user_info(&self, user_id: &str) -> Result<FullUserInfo> {
        let user = self.get_user_by_id(user_id)?;

        let sessions = self.get_sessions_by_user(user_id)?;

        Ok(FullUserInfo { user, sessions })
    }
}
