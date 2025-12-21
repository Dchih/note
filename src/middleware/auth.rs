use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use crate::utils::JwtUtil;

pub struct Auth {
    pub jwt_secret: String,
}

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware {
            service: Rc::new(service),
            jwt_secret: self.jwt_secret.clone(),
        })
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let jwt_secret = self.jwt_secret.clone();

        Box::pin(async move {
            // 从 Header 提取 Token
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            let token = match auth_header {
                Some(h) if h.starts_with("Bearer ") => &h[7..],
                _ => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "code": 401,
                            "message": "Missing token"
                        }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            // 验证 Token
            match JwtUtil::verify_token(token, &jwt_secret) {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    service.call(req).await.map(|res| res.map_into_left_body())
                }
                Err(_) => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "code": 401,
                            "message": "Invalid token"
                        }));
                    Ok(req.into_response(response).map_into_right_body())
                }
            }
        })
    }
}