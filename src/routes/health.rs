use actix_web::{web, HttpResponse};
use crate::{error::AppError};

async fn headth_check () -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

async fn test_error (path: web::Path<String>) -> Result<HttpResponse, AppError>{
    let error_type = path.into_inner();

    match error_type.as_str() {
        "404" => Err(AppError::NotFound("资源不存在".to_string())),
        "400" => Err(AppError::BadRequest("参数错误".to_string())),
        "500" => Err(AppError::Internal("数据库炸了".to_string())),
        _ => Ok(HttpResponse::Ok().body("No Error"))
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(headth_check))
        .route("/test-error/{type}", web::get().to(test_error));
}