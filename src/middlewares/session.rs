use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::cookie::Key;

pub fn session_middleware(secret_key: String) -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(
        CookieSessionStore::default(),
        Key::from(secret_key.as_bytes()),
    )
    .cookie_content_security(actix_session::config::CookieContentSecurity::Private)
    .cookie_name("token".to_string())
    .cookie_http_only(false)
    .build()
}
