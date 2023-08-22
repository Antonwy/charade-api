use diesel::{Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Identifiable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(user_id, session_id))]
pub struct UsersSession {
    pub user_id: String,
    pub session_id: String,
}
