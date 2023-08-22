use actix_session::Session;
use actix_web::{
    get, post,
    web::{self, Data},
    HttpResponse, Responder,
};

use crate::{
    extractors::user_id::UserId,
    models::{
        custom_api_errors::ApiError,
        user::{NewUser, NewUserDto, User},
    },
    AppContext, Response,
};

pub const SESSION_USER_ID: &str = "user_id";

#[post("/auth")]
async fn authenticate(
    session: Session,
    ctx: web::Data<AppContext>,
    user_body: web::Json<NewUserDto>,
) -> Result<impl Responder, ApiError> {
    let user_body = user_body.into_inner();

    let user_id = session.get::<String>(SESSION_USER_ID)?;

    let user: User;

    if let Some(user_id) = user_id {
        user = ctx.db.update_user(NewUser {
            id: user_id,
            name: user_body.name,
        })?;
    } else {
        user = ctx.db.create_user(NewUser {
            id: uuid::Uuid::new_v4().to_string(),
            name: user_body.name,
        })?;

        session.insert(SESSION_USER_ID, user.id.clone())?;
    }

    Ok(HttpResponse::Ok().json(user))
}

#[get("/account")]
async fn get_account(user_id: UserId, ctx: Data<AppContext>) -> Result<impl Responder, ApiError> {
    let user = ctx.db.get_user_by_id(user_id.into())?;

    Ok(HttpResponse::Ok().json(user))
}

#[post("/logout")]
async fn logout(session: Session) -> Result<impl Responder, ApiError> {
    session
        .remove(SESSION_USER_ID)
        .ok_or(ApiError::bad_request("Not logged in".to_string()))?;

    Ok(HttpResponse::Ok().json(Response {
        message: "Logged out".to_string(),
    }))
}

pub fn config(config: &mut web::ServiceConfig) {
    config
        .service(authenticate)
        .service(get_account)
        .service(logout);
}
