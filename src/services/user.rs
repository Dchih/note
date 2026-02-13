use sqlx::{MySqlPool};
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::error::AppError;
use crate::models::{User, RegisterRequest};

pub struct UserService;

impl UserService {
  pub async fn register(pool: &MySqlPool, data: RegisterRequest) -> Result<User, AppError> {
    let exists = sqlx::query_scalar::<_, i64>(
      "SELECT COUNT(*) FROM users WHERE username = ?"
    )
      .bind(&data.username)
      .fetch_one(pool)
      .await
      .map_err(|e| AppError::Internal(e.to_string()))?;

    if exists > 0 {
      return Err(AppError::BadRequest("用户名已存在".to_string()));
    }

    let hashed_password = hash(&data.password, DEFAULT_COST)
      .map_err(|e| AppError::Internal(e.to_string()))?;

    let result = sqlx::query(
      "INSERT INTO users (username, password, email) VALUES (?, ?, ?)"
    )
      .bind(&data.username)
      .bind(&hashed_password)
      .bind(&data.email)
      .execute(pool)
      .await
      .map_err(|e| AppError::Internal(e.to_string()))?;

    Self::find_by_id(pool, result.last_insert_id() as i64).await
  }

  pub async fn find_by_id(pool: &MySqlPool, id:i64) -> Result<User, AppError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
      .bind(id)
      .fetch_optional(pool)
      .await
      .map_err(|e| AppError::Internal(e.to_string()))?
      .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))
  }

  pub async fn find_by_username(pool: &MySqlPool, username: &str) -> Result<User, AppError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
      .bind(username)
      .fetch_optional(pool)
      .await
      .map_err(|e| AppError::Internal(e.to_string()))?
      .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))
  }

  pub fn verify_password(password: &str, hashed: &str) -> Result<bool, AppError> {
    verify(password, hashed)
      .map_err(|e| AppError::Internal(e.to_string()))
  }
}