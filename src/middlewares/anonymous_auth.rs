use std::future::{ready, Ready};

use actix_session::SessionExt;
use actix_web::{
    body::MessageBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, Result,
};
use futures_util::future::LocalBoxFuture;

use crate::{models::user::NewUser, routes::SESSION_USER_ID, AppContext};

#[derive(Default, Clone)]
pub struct AnonymousAuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AnonymousAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = InnerAnonymousAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InnerAnonymousAuthMiddleware { service: service }))
    }
}

#[doc(hidden)]
pub struct InnerAnonymousAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for InnerAnonymousAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let session = req.get_session();
        let ctx = req.app_data::<web::Data<AppContext>>().unwrap();

        match session.get::<String>(SESSION_USER_ID) {
            Ok(Some(_)) => {}
            Ok(None) => {
                log::info!("No UserId found in session");
                let new_user = NewUser {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: None,
                };

                let user = ctx.db.create_user(new_user);

                if let Ok(user) = user {
                    log::info!("Created new user: {:?}", user);
                    session.insert(SESSION_USER_ID, user.id.clone()).unwrap();
                }
            }
            Err(e) => {
                log::info!("Error getting UserId from session: {}", e);
            }
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            Ok(res)
        })
    }
}
