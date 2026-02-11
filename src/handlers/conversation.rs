use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::env;

use crate::error::AppError;
use crate::middleware::Auth;
use crate::models::MemberRole;
use crate::services::ConversationServices;
use crate::utils::Claims;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConversationReq {
  name: Option<String>,
  member_ids: Vec<i64>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMemberReq {
  user_id: i64
}

pub async fn create(pool: web::Data<MySqlPool>, body: web::Json<CreateConversationReq>, claims: web::ReqData<Claims>) -> Result<HttpResponse, AppError> {
  let CreateConversationReq {name, member_ids} = body.into_inner();
  let conv = ConversationServices::create(pool.get_ref(), claims.sub, name, member_ids).await?;
  Ok(HttpResponse::Created().json(conv))
}

pub async fn list(pool: web::Data<MySqlPool>, claims: web::ReqData<Claims>) -> Result<HttpResponse, AppError> {
  let list = ConversationServices::get_user_conversations(pool.get_ref(), claims.sub).await?;
  Ok(HttpResponse::Ok().json(list))
}

pub async fn add_member(pool: web::Data<MySqlPool>, body: web::Json<AddMemberReq>, path: web::Path<i64>) -> Result<HttpResponse, AppError> {
  let AddMemberReq {user_id} = body.into_inner();
  let conversation_id = path.into_inner();
  let result = ConversationServices::add_member(pool.get_ref(), user_id, conversation_id, MemberRole::Member).await?;
  Ok(HttpResponse::Ok().json(result))
}


pub fn configure(cfg: &mut web::ServiceConfig) {
  let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is required");

  cfg.service(
    web::scope("/conversations")
    .wrap(Auth {jwt_secret })
    .route("", web::post().to(create))
    .route("", web::get().to(list))
    .route("/{conversation_id}/members", web::post().to(add_member))
  );
}