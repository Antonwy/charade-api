use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, Identifiable, Serialize)]
#[diesel(table_name = crate::schema::words)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(word, session_id))]
pub struct Word {
    pub word: String,
    pub created_at: NaiveDateTime,
    pub session_id: String,
    pub user_id: String,
}

#[derive(Insertable, Debug, Deserialize)]
#[diesel(table_name = crate::schema::words)]
pub struct NewWord {
    pub word: String,
    pub session_id: String,
    pub user_id: String,
}
