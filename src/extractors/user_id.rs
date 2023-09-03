use std::pin::Pin;

use actix_session::SessionExt;
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures_util::Future;

use crate::{
    models::{custom_api_errors::ApiError, user::NewUser},
    routes::SESSION_USER_ID,
    AppContext,
};

#[derive(Debug, Clone)]
pub struct UserId(pub String);

impl UserId {
    async fn extract(req: HttpRequest) -> Result<UserId, ApiError> {
        let session = req.get_session();
        let ctx = req
            .app_data::<web::Data<AppContext>>()
            .ok_or(ApiError::internal(
                "Could not get AppContext from request".to_string(),
            ))?;

        let user_id_opt = session.get::<String>("user_id")?;

        match user_id_opt {
            Some(id) => Ok(id.into()),
            None => {
                let new_user = NewUser {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: None,
                };

                let user = ctx.db.create_user(new_user).await?;

                session.insert(SESSION_USER_ID, user.id.clone())?;

                Ok(user.id.into())
            }
        }
    }
}

impl From<String> for UserId {
    fn from(user_id: String) -> Self {
        Self(user_id)
    }
}

impl From<UserId> for String {
    fn from(user_id: UserId) -> Self {
        user_id.0
    }
}

impl From<&str> for UserId {
    fn from(user_id: &str) -> Self {
        Self(user_id.to_string())
    }
}

impl FromRequest for UserId {
    type Error = ApiError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req_clone = req.clone();

        Box::pin(async {
            match UserId::extract(req_clone).await {
                Ok(user_id) => Ok(user_id),
                Err(e) => Err(e),
            }
        })
    }
}
