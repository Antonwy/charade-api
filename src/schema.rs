// @generated automatically by Diesel CLI.

diesel::table! {
    sessions (id) {
        #[max_length = 50]
        id -> Varchar,
        public -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 50]
        name -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users_sessions (user_id, session_id) {
        #[max_length = 36]
        user_id -> Varchar,
        #[max_length = 20]
        session_id -> Varchar,
    }
}

diesel::joinable!(users_sessions -> sessions (session_id));
diesel::joinable!(users_sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    sessions,
    users,
    users_sessions,
);
