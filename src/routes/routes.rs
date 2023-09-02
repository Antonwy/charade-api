use actix_web::web;

use super::{auth, sessions, ws};

pub fn config(config: &mut web::ServiceConfig) {
    config
        .service(
            web::scope("/api")
                .configure(sessions::config)
                .configure(auth::config),
        )
        .configure(ws::config);
}
