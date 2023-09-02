use std::time::Instant;

use actix::Addr;
use actix_session::Session;
use actix_web::{
    cookie::{Cookie, CookieJar, Key},
    get, web, HttpRequest, HttpResponse, ResponseError,
};
use actix_web_actors::ws;
use serde::Deserialize;

use crate::{
    models::custom_api_errors::ApiError,
    websocket::{server, session},
    AppContext,
};

#[derive(Debug, Deserialize)]
struct ConnectWebsocketPathParams {
    session_id: String,
    user_cookie: String,
}

#[derive(Debug, Deserialize)]
struct UserTokenBody {
    user_id: String,
}

#[get("/ws/{session_id}/{user_cookie}")]
async fn connect_websocket(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::CharadeServer>>,
    ctx: web::Data<AppContext>,
    path_params: web::Path<ConnectWebsocketPathParams>,
) -> Result<HttpResponse, actix_web::Error> {
    let cookie = Cookie::parse(format!("token={}", path_params.user_cookie.clone())).map_err(
        |_: actix_web::cookie::ParseError| {
            actix_web::Error::from(ApiError::internal("Could not parse cookie".to_string()))
        },
    )?;

    let mut jar = CookieJar::new();

    jar.add_original(cookie);

    let secret_key = Key::from(ctx.cookie_secret.as_bytes());

    let cookie_json_string = jar.private(&secret_key).get("token").ok_or_else(|| {
        actix_web::Error::from(ApiError::internal("Could not decrypt cookie".to_string()))
    })?;

    let cookie_json = serde_json::from_str::<UserTokenBody>(cookie_json_string.value())?;

    let user_id = serde_json::from_str::<String>(&cookie_json.user_id)?;

    ws::start(
        session::WsCharadeSession {
            id: user_id,
            session_id: path_params.session_id.clone(),
            hb: Instant::now(),
            server: srv.get_ref().clone(),
            db: ctx.db.clone(),
        },
        &req,
        stream,
    )
}

pub fn config(config: &mut web::ServiceConfig) {
    config.service(connect_websocket);
}
