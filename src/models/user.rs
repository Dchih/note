use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
  pub id: i64,
  pub username: String,
  #[serde(skip_serializing)]
  pub password_hash: String,
  pub email: Option<String>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>
}

#[derive(Debug, Deserialize)]
pub struct RegisterReuqest {
  pub username: String,
  pub password: String,
  pub email: Option<String>,
}