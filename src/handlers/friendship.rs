use std::env;

use actix_web::{ HttpResponse, web};
use serde::{ Deserialize, Serialize };
use sqlx::MySqlPool;

use crate::{error::AppError, middleware::Auth, services::FriednShipService, utils::Claims};


#[derive(Debug, Serialize, Deserialize)]
pub struct FriendShipReq {
  receiver_id: i64
}

async fn send_friendship_request(pool: web::Data<MySqlPool>, body: web::Json<FriendShipReq>, claims: web::ReqData<Claims>) -> Result<HttpResponse, AppError> {
  let FriendShipReq { receiver_id} = body.into_inner();
  let result = FriednShipService::send_request(pool.get_ref(), claims.sub, receiver_id).await?;

  Ok(HttpResponse::Ok().json(result))
}

async fn accept_friendship(pool: web::Data<MySqlPool>, path: web::Path<i64>, claims: web::ReqData<Claims>) -> Result<HttpResponse, AppError> {
  let friendship_id = path.into_inner();
  let result = FriednShipService::accept(pool.get_ref(), friendship_id, claims.sub).await?;

  Ok(HttpResponse::Ok().json(result))
}

async fn reject_friendship(pool: web::Data<MySqlPool>, path: web::Path<i64>, claims: web::ReqData<Claims>) -> Result<HttpResponse, AppError> {
  let friendship_id = path.into_inner();
  let result = FriednShipService::reject(pool.get_ref(), friendship_id, claims.sub).await?;

  Ok(HttpResponse::Ok().json(result))
}

async fn list_pending(pool: web::Data<MySqlPool>, claims: web::ReqData<Claims>) -> Result<HttpResponse, AppError> {
  let result = FriednShipService::list_pending(pool.get_ref(), claims.sub).await?;
  Ok(HttpResponse::Ok().json(result))
}

async fn list_friends(pool: web::Data<MySqlPool>, claims: web::ReqData<Claims>) -> Result<HttpResponse, AppError> {
  let result = FriednShipService::list_friends(pool.get_ref(), claims.sub).await?;
  Ok(HttpResponse::Ok().json(result))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
  let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is required");

  cfg.service(
    web::scope("/friendships")
    .wrap(Auth { jwt_secret })
    .route("", web::post().to(send_friendship_request))
    .route("", web::get().to(list_friends))
    .route("/{id}/accept", web::post().to(accept_friendship))
    .route("/{id}/reject", web::post().to(reject_friendship))
    .route("/pending", web::get().to(list_pending))
  );
}
