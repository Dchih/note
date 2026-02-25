use actix_web::web;
use serde::{ Deserialize, Serialize };
use sqlx::MySqlPool;

use crate::{error::AppError, services::FriednShipService, utils::Claims};


#[derive(Debug, Serialize, Deserialize)]
pub struct FriendShipReq {
  receiver_id: i64
}

async fn send_friendship_request(pool: web::Data<MySqlPool>, body: web::Json<FriendShipReq>, claims: web::ReqData<Claims>) -> Result<(), AppError> {
  let FriendShipReq { receiver_id } = body.into_inner();
  let fs = FriednShipService::send_request(pool.get_ref(), claims.sub, receiver_id);
  Ok(())
}

