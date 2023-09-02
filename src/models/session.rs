use super::{user::User, word::Word};
use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, Identifiable, Serialize, Clone)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub public: bool,
    pub created_at: NaiveDateTime,
    pub admin_user_id: String,
}

#[derive(Insertable, Debug, Deserialize)]
#[diesel(table_name = crate::schema::sessions)]
pub struct NewSession {
    pub id: String,
    pub public: bool,
    pub admin_user_id: String,
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub session: Session,
    pub words: Vec<Word>,
    pub users: Vec<User>,
}

#[derive(Debug, Serialize)]
pub struct SessionInfoPersonal {
    pub my_id: String,
    pub session: Session,
    pub my_words: Vec<String>,
    pub users: Vec<User>,
    pub number_of_words: u32,
}
