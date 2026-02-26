use sqlx::MySqlPool;

use crate::{error::AppError, models::FriendShip};
use crate::models::FriendShipStatus;

pub struct FriednShipService;

impl FriednShipService {
  pub async fn send_request(pool: &MySqlPool, requester_id: i64, receiver_id: i64) -> Result<bool, AppError> {
    if requester_id == receiver_id {
      return Err(AppError::Unauthorized("不能添加自己为好友".to_string()))
    }
    let result = sqlx::query_as::<_, FriendShip>(
      "SELECT * FROM friendships WHERE requester_id = ? AND receiver_id = ?"
    )
    .bind(requester_id)
    .bind(receiver_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    match result {
      Some(record) => {
        match record.status {
          FriendShipStatus::Accepted => {
            return Err(AppError::Unauthorized("对方已是您的好友".to_string()))
          },
          FriendShipStatus::Pending => {
            return Err(AppError::Unauthorized("不能重复添加".to_string()))
          }
          FriendShipStatus::Rejected => {
            sqlx::query(
              "UPDATE friendships SET status = 'pending' WHERE id = ? AND requester_id = ?"
            )
            .bind(record.id)
            .bind(requester_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
          }
        }
      },
      None => {
        sqlx::query(
          "INSERT INTO friendships (requester_id, receiver_id, status) VALUE (?, ?, 'pending')"
        )
        .bind(requester_id)
        .bind(receiver_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
      }
    }
    Ok(true)
  }

  pub async fn accept(pool: &MySqlPool, friendship_id: i64, user_id: i64) -> Result<bool, AppError> {
      let result = sqlx::query(
          "UPDATE friendships SET status = 'accepted' WHERE id = ? AND receiver_id = ?" 
        )
        .bind(friendship_id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        
    if result.rows_affected() == 0 {
      return Err(AppError::Unauthorized("只有接收方可以接收好友请求".to_string()))
    }
    return Ok(true)
  }

  pub async fn reject(pool: &MySqlPool, friendship_id: i64, receiver_id: i64) -> Result<bool, AppError> {
    sqlx::query(
      "UPDATE friendships SET status = 'rejected' WHERE id = ? AND receiver_id = ?"
    )
    .bind(friendship_id)
    .bind(receiver_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(true)
  }

  // 查询所有 receiver_id = user_id 且 status = 'pending' 的记录
  // 返回 Result<Vec<FriendShip>, AppError>
  pub async fn list_pending(pool: &MySqlPool, user_id: i64) -> Result<Vec<FriendShip>, AppError> {
    // todo!()
    let result = sqlx::query_as::<_, FriendShip>(
      "SELECT * FROM friendships WHERE receiver_id = ? AND status = 'pending'"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(result)
  }

  #[allow(dead_code)]
  pub async fn is_friend(pool: &MySqlPool, requester_id: i64, receiver_id: i64) -> Result<bool, AppError> {
    let result = sqlx::query_as::<_, FriendShip>(
      "SELECT * from friendships WHERE status = 'accepted' AND ((requester_id = ? AND receiver_id = ?) OR (requester_id = ? AND receiver_id = ?))"
    )
    .bind(requester_id)
    .bind(receiver_id)
    .bind(receiver_id)
    .bind(requester_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(_record) = result {
      return Ok(true)
    } else {
      return Ok(false)
    }
  }

}