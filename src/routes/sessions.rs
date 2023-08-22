use actix_web::{
    get, post,
    web::{self, Data, Path},
    HttpResponse, Responder, Result,
};

use crate::{
    extractors::user_id::UserId,
    models::{custom_api_errors::ApiError, session::NewSession},
    AppContext,
};

#[post("")]
async fn create_session(
    ctx: Data<AppContext>,
    new_session: web::Json<NewSession>,
    user_id: UserId,
) -> Result<impl Responder, ApiError> {
    let created_session = ctx
        .db
        .create_session(new_session.into_inner(), user_id.into())?;
    Ok(HttpResponse::Ok().json(created_session))
}

#[get("")]
async fn get_sessions(ctx: Data<AppContext>) -> Result<impl Responder, ApiError> {
    let sessions = ctx.db.get_sessions()?;
    Ok(HttpResponse::Ok().json(sessions))
}

#[get("/{session_id}")]
async fn get_session(
    session_id: Path<String>,
    ctx: Data<AppContext>,
) -> Result<impl Responder, ApiError> {
    let session = ctx.db.get_session_by_id(session_id.into_inner())?;
    Ok(HttpResponse::Ok().json(session))
}

#[post("/{session_id}/join")]
async fn join_session(
    user_id: UserId,
    ctx: Data<AppContext>,
    session_id: Path<String>,
) -> Result<impl Responder, ApiError> {
    let session = ctx
        .db
        .join_session(session_id.into_inner(), user_id.into())?;

    Ok(HttpResponse::Ok().json(session))
}

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/sessions")
            .service(create_session)
            .service(get_sessions)
            .service(get_session)
            .service(join_session),
    );
}
