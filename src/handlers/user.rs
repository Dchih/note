use std::env;

use actix_web::{HttpResponse, web::{self, ServiceConfig}};
use sqlx::MySqlPool;

use crate::{error::AppError, middleware::Auth, services::UserService};

#[derive(serde::Deserialize)]
struct SearchQuery {
    q: String,
}

#[allow(private_interfaces)]
pub async fn search_user(pool: web::Data<MySqlPool>, query: web::Query<SearchQuery>) -> Result<HttpResponse, AppError> {
  let q = query.into_inner().q;
  let result = UserService::search(pool.get_ref(), &q).await?;
  Ok(HttpResponse::Ok().json(result))
}

pub async fn get_user(
  pool: web::Data<MySqlPool>,
  path: web::Path<i64>
) -> Result<HttpResponse, AppError> {
  let id = path.into_inner();
  let user = UserService::find_by_id(pool.get_ref(), id).await?;

  Ok(HttpResponse::Ok().json(user))
}

pub fn configure(cfg: &mut ServiceConfig) {
  let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is required");

  cfg.service(
    web::scope("/users")
    .wrap(Auth { jwt_secret })
    .route("/search", web::get().to(search_user))
    .route("/{id}", web::get().to(get_user))
  );
}