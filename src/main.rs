use actix_web::{
    error::JsonPayloadError,
    middleware,
    web::{self, Data},
    App, HttpResponse, HttpServer, Result,
};
use models::custom_api_errors::ApiError;
use repositories::database::Database;
use routes::routes::config;
use serde::Serialize;
use utils::{db, envs};

mod extractors;
mod middlewares;
mod models;
mod repositories;
mod routes;
mod schema;
mod utils;

#[derive(Debug, Serialize)]
pub struct Response {
    pub message: String,
}

#[derive(Clone)]
pub struct AppContext {
    pub db: Database,
}

async fn not_found() -> Result<HttpResponse> {
    let response = Response {
        message: "Not found!".to_string(),
    };

    Ok(HttpResponse::NotFound().json(response))
}

fn json_config() -> web::JsonConfig {
    web::JsonConfig::default().error_handler(|err, _req| match err {
        JsonPayloadError::ContentType => actix_web::error::InternalError::from_response(
            err,
            HttpResponse::UnsupportedMediaType().json(ApiError {
                status: 415,
                message: "Unsupported media type".to_string(),
            }),
        )
        .into(),
        JsonPayloadError::Deserialize(json_err) if json_err.is_data() => {
            actix_web::error::InternalError::from_response(
                "idk",
                HttpResponse::BadRequest().json(ApiError {
                    status: 400,
                    message: json_err.to_string(),
                }),
            )
            .into()
        }
        _ => actix_web::error::InternalError::from_response(
            "idk",
            HttpResponse::BadRequest().json(ApiError {
                status: 400,
                message: err.to_string(),
            }),
        )
        .into(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db = Database::init();

    db::run_migrations(&mut *db.connection());

    let app_context = Data::new(AppContext { db });

    let cookies_secret = envs::cookie_secret();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .app_data(json_config())
            .app_data(app_context.clone())
            .configure(config)
            .wrap(middlewares::session::session_middleware(
                cookies_secret.to_owned(),
            ))
            .default_service(web::route().to(not_found))
            .wrap(middleware::NormalizePath::trim())
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 3001))?
    .run()
    .await
}
