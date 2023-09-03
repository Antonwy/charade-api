use actix::Actor;
use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Result,
};
use api_configs::{cors_config, json_config, json_validator_config};
use repositories::{cache::Cache, database::Database};
use routes::config;
use serde::Serialize;
use utils::{db, envs};
use websocket::server;

mod api_configs;
mod extractors;
mod middlewares;
mod models;
mod repositories;
mod routes;
mod schema;
mod utils;
mod websocket;

#[derive(Debug, Serialize)]
pub struct Response {
    pub message: String,
}

#[derive(Clone)]
pub struct AppContext {
    pub db: Database,
    pub cache: Cache,
    pub cookie_secret: String,
}

async fn not_found() -> Result<HttpResponse> {
    let response = Response {
        message: "Not found!".to_string(),
    };

    Ok(HttpResponse::NotFound().json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db = Database::init();
    let cache = Cache::init();

    db::run_migrations(&mut *db.connection().expect("Could not get database connection"));

    let cookies_secret = envs::cookie_secret();

    let app_context = Data::new(AppContext {
        db: db.clone(),
        cache: cache.clone(),
        cookie_secret: cookies_secret.clone(),
    });

    let server = server::CharadeServer::new(db, cache).start();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .app_data(json_config())
            .app_data(json_validator_config())
            .app_data(app_context.clone())
            .app_data(web::Data::new(server.clone()))
            .configure(config)
            .wrap(middlewares::session::session_middleware(
                cookies_secret.to_owned(),
            ))
            .default_service(web::route().to(not_found))
            .wrap(cors_config())
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 3001))?
    .run()
    .await
}
