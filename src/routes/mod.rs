mod auth;
mod sessions;
mod ws;

use actix_web::web;
pub use auth::SESSION_USER_ID;

pub fn config(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api")
                .configure(sessions::config)
                .configure(auth::config),
        )
        .configure(ws::config);
}
