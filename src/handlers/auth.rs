use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::{MySqlPool};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::services::UserService;
use crate::utils::JwtUtil;
use crate::models::RegisterReuqest;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

async fn login(
    pool: web::Data<MySqlPool>,
    config: web::Data<AppConfig>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let user = UserService::find_by_username(pool.get_ref(), &body.username).await?;

    let is_valid = UserService::verify_password(&body.password, &user.password_hash)?;
    if !is_valid {
        return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
    }

    let token = JwtUtil::generate_token(1, &config.jwt_secret)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "token": token,
        "data": user
    })))
}

async fn register(
    pool: web::Data<MySqlPool>,
    body: web::Json<RegisterReuqest>
) -> Result<HttpResponse, AppError> {
    let user = UserService::register(pool.get_ref(), body.into_inner()).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "code": 201,
        "message": "注册成功",
        "data": user
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/login", web::post().to(login))
        .route("/register", web::post().to(register));
}