use actix_web::{
    get, post,
    web::{self, Data, Path},
    HttpResponse, Responder, Result,
};
use actix_web_validator::Json;

use crate::{
    extractors::user_id::UserId,
    models::{
        custom_api_errors::ApiError,
        dtos::{session::NewSessionDto, word::NewWordDto},
        session::{NewSession, SessionInfoPersonal},
        word::NewWord,
    },
    AppContext,
};

#[post("")]
async fn create_session(
    ctx: Data<AppContext>,
    new_session: Json<NewSessionDto>,
    user_id: UserId,
) -> Result<impl Responder, ApiError> {
    let new_session = NewSession {
        id: new_session.id.clone(),
        public: new_session.public.unwrap_or(false),
        admin_user_id: user_id.0,
    };

    let created_session = web::block(move || ctx.db.create_session(new_session))
        .await?
        .map_err(|e| match e {
            ApiError::UniqueViolation { .. } => ApiError::UniqueViolation {
                message: "Session already exists".to_string(),
            },
            _ => e,
        })?;

    Ok(HttpResponse::Ok().json(created_session))
}

#[get("")]
async fn get_sessions(ctx: Data<AppContext>) -> Result<impl Responder, ApiError> {
    let sessions = web::block(move || ctx.db.get_sessions()).await??;
    Ok(HttpResponse::Ok().json(sessions))
}

#[get("/personal")]
async fn get_all_personal_sessions(
    user_id: UserId,
    ctx: Data<AppContext>,
) -> Result<impl Responder, ApiError> {
    let sessions = web::block(move || ctx.db.get_sessions_by_user(&user_id.0)).await??;

    Ok(HttpResponse::Ok().json(sessions))
}

#[get("/{session_id}")]
async fn get_session(
    session_id: Path<String>,
    ctx: Data<AppContext>,
) -> Result<impl Responder, ApiError> {
    let session = web::block(move || ctx.db.get_session_by_id(&session_id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(session))
}

#[get("/{session_id}/personal")]
async fn get_personal_session(
    session_id: Path<String>,
    ctx: Data<AppContext>,
    user_id: UserId,
) -> Result<impl Responder, ApiError> {
    let session_info =
        web::block(move || ctx.db.get_session_info(&session_id.into_inner())).await??;

    Ok(HttpResponse::Ok().json(SessionInfoPersonal {
        my_id: user_id.clone().0,
        session: session_info.session,
        my_words: session_info
            .words
            .iter()
            .filter(|word| word.user_id == user_id.0)
            .map(|word| word.word.clone())
            .collect(),
        users: session_info.users,
        number_of_words: session_info.words.len() as u32,
    }))
}

#[post("/{session_id}/join")]
async fn join_session(
    user_id: UserId,
    ctx: Data<AppContext>,
    session_id: Path<String>,
) -> Result<impl Responder, ApiError> {
    let user_id_clone = user_id.clone();
    let session_id_clone = session_id.clone();
    let db_cloned = ctx.db.clone();

    let result = web::block(move || db_cloned.join_session(&session_id_clone, &user_id_clone.0))
        .await?
        .map_err(|e| match e {
            ApiError::NotFound { .. } | ApiError::ForeignKeyViolation { .. } => {
                ApiError::NotFound {
                    message: "Session not found".to_string(),
                }
            }
            _ => e,
        });

    match result {
        Err(ApiError::UniqueViolation { .. }) => {
            let session = web::block(move || ctx.db.get_session_by_id(&session_id)).await??;

            Ok(HttpResponse::Ok().json(session))
        }
        Err(e) => Err(e),
        Ok(session) => Ok(HttpResponse::Ok().json(session)),
    }
}

#[post("/{session_id}/words")]
async fn add_word_to_session(
    ctx: Data<AppContext>,
    session_id: Path<String>,
    new_word: Json<NewWordDto>,
    user_id: UserId,
) -> Result<impl Responder, ApiError> {
    let new_word = NewWord {
        word: new_word.word.to_owned(),
        session_id: session_id.to_owned(),
        user_id: user_id.into(),
    };

    let word = web::block(move || ctx.db.add_word_to_session(new_word)).await??;

    Ok(HttpResponse::Ok().json(word))
}

pub fn config(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/sessions")
            .service(create_session)
            .service(get_sessions)
            .service(get_all_personal_sessions)
            .service(get_session)
            .service(get_personal_session)
            .service(join_session)
            .service(add_word_to_session),
    );
}
