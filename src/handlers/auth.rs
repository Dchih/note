use actix_web::{web, HttpResponse};
use serde::Deserialize;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::utils::JwtUtil;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

async fn login(
    config: web::Data<AppConfig>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    // 临时写死，后续改成查数据库
    if body.username == "admin" && body.password == "123456" {
        let token = JwtUtil::generate_token(1, &config.jwt_secret)?;
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "token": token
        })))
    } else {
        Err(AppError::Unauthorized("用户名或密码错误".to_string()))
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login));
}