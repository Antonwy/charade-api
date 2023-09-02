use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::user::User;

#[derive(Queryable, Selectable, Debug, Identifiable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(user_id, session_id))]
pub struct UsersSession {
    pub user_id: String,
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionUser {
    pub user: User,
    pub number_of_words: u16,
}
