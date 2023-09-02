use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use super::session::Session;

#[derive(Queryable, Selectable, Debug, Clone, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct FullUserInfo {
    pub user: User,
    pub sessions: Vec<Session>,
}
