use actix_session::Session;
use actix_web::{
    get, post,
    web::{self, Data},
    HttpResponse, Responder,
};
use actix_web_validator::Json;

use crate::{
    extractors::user_id::UserId,
    models::{
        custom_api_errors::ApiError,
        dtos::user::NewUserDto,
        user::{NewUser, User},
    },
    AppContext, Response,
};

pub const SESSION_USER_ID: &str = "user_id";

#[post("/auth")]
async fn authenticate(
    session: Session,
    ctx: web::Data<AppContext>,
    user_body: Json<NewUserDto>,
) -> Result<impl Responder, ApiError> {
    let user_body = user_body.into_inner();

    let user_id = session.get::<String>(SESSION_USER_ID)?;

    let user: User;

    if let Some(user_id) = user_id {
        user = web::block(move || {
            ctx.db.update_user(NewUser {
                id: user_id,
                name: user_body.name.map(|name| name.trim().to_string()),
            })
        })
        .await??;
    } else {
        user = web::block(move || {
            ctx.db.create_user(NewUser {
                id: uuid::Uuid::new_v4().to_string(),
                name: user_body.name.map(|name| name.trim().to_string()),
            })
        })
        .await??;

        session.insert(SESSION_USER_ID, user.id.clone())?;
    }

    Ok(HttpResponse::Ok().json(user))
}

#[get("/account")]
async fn get_account(user_id: UserId, ctx: Data<AppContext>) -> Result<impl Responder, ApiError> {
    let user = web::block(move || ctx.db.get_user_by_id(&user_id.0)).await??;

    Ok(HttpResponse::Ok().json(user))
}

#[post("/logout")]
async fn logout(session: Session) -> Result<impl Responder, ApiError> {
    session
        .remove(SESSION_USER_ID)
        .ok_or(ApiError::BadRequest {
            message: "Not logged in".to_string(),
        })?;

    Ok(HttpResponse::Ok().json(Response {
        message: "Logged out".to_string(),
    }))
}

#[get("/account/full")]
async fn get_full_user_info(
    user_id: UserId,
    ctx: Data<AppContext>,
) -> Result<impl Responder, ApiError> {
    let user = web::block(move || ctx.db.get_full_user_info(&user_id.0)).await??;

    Ok(HttpResponse::Ok().json(user))
}

pub fn config(config: &mut web::ServiceConfig) {
    config
        .service(authenticate)
        .service(get_account)
        .service(logout)
        .service(get_full_user_info);
}
