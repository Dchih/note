use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, sqlx::Type, Serialize)]
#[sqlx(type_name = "ENUM", rename_all = "lowercase")]
pub enum FriendShipStatus {
  Pending,
  Accepted,
  Rejected
}

#[derive(Debug, FromRow, Serialize)]
pub struct FriendShip {
  pub id: i64,
  pub requester_id: i64,
  pub receiver_id: i64,
  pub status: FriendShipStatus,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>
}