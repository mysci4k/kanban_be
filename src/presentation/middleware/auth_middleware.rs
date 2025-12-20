use crate::shared::error::ApplicationError;
use actix_identity::Identity;
use actix_web::{
    Error, FromRequest, HttpMessage,
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use uuid::Uuid;

pub struct RequireAuth;

impl<S, B> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequireAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let path = req.path().to_string();

            if path == "/api/"
                || (path.starts_with("/api/auth") && path != "/api/auth/logout")
                || path.starts_with("/scalar")
            {
                return service.call(req).await;
            }

            if let Some(identity) = Identity::from_request(req.request(), &mut Payload::None)
                .await
                .ok()
                && let Ok(user_id_str) = identity.id()
                && let Ok(user_id) = Uuid::parse_str(&user_id_str)
            {
                req.extensions_mut().insert(user_id);

                return service.call(req).await;
            }

            Err(ApplicationError::Unauthorized {
                message: "Authentication required".to_string(),
            }
            .into())
        })
    }
}
