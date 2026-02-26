use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct FieldError {
    pub field: String,
    pub code: String,
}


#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
    Unauthorized(String),
    #[allow(dead_code)]
    Validation(Vec<FieldError>)
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Error: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Validation(errors) => write!(f, "Validation Failed: {} errors", errors.len()),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(serde_json::json!({
                    "code": 404,
                    "message": msg
                }))
            }
            AppError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "code": 400,
                    "message": msg
                }))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "code": 500,
                    "message": "Internal Server Error"  // 不暴露内部细节
                }))
            }
            AppError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "code": 401,
                    "message": msg
                }))
            }
            AppError::Validation(errors) => {
                HttpResponse::UnprocessableEntity().json(serde_json::json!({
                    "msg": "Validation Failed",
                    "err": errors
                }))
            }
        }
    }
}