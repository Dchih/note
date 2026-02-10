use sqlx::MySqlPool;
use sqlx::prelude::FromRow;
use crate::error::AppError;
use crate::models::{ MemberRole, ConversationType};

#[derive(FromRow)]
pub struct ConversationRes {
  pub id: i64,
  pub conversation_type: ConversationType,
  pub name: Option<String>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub is_deleted: bool
}

pub struct ConversationServices;

impl ConversationServices {
  pub async fn create(pool: &MySqlPool, user_id: i64, name: Option<String>, member_ids: Vec<i64> ) -> Result<i64, AppError> {
    let members_num = member_ids.len();
    let mut tx = pool.begin().await.map_err(|e| AppError::Internal(e.to_string()))?;

    let conversation_result = sqlx::query(
      "INSERT INTO conversations (conversation_type, name, is_deleted) VALUES (0, ?, false)"
    )
    .bind(&name)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    sqlx::query(
      "INSERT INTO conversation_member (conversation_id, user_id, role) VALUES (?, ?, ?)"
    )
    .bind(conversation_result.last_insert_id())
    .bind(&user_id)
    .bind(if members_num > 1 { MemberRole::Owner } else { MemberRole::Member})
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    for member in &member_ids {
      sqlx::query(
        "INSERT INTO conversation_member (conversation_id, user_id, role) VALUES (?, ?, ?)"
      )
      .bind(conversation_result.last_insert_id())
      .bind(member)
      .bind(MemberRole::Member)
      .execute(&mut *tx)
      .await
      .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    tx.commit().await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(conversation_result.last_insert_id().try_into().unwrap())
  }

  /**
   * id: conversation_id
   * 查询需要用fetch_optional fetch_one
   * 写用 execute
   */
  pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<ConversationRes, AppError> {
    sqlx::query_as(
      "SELECT * FROM conversation WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("会话不存在".to_string()))
  }

  pub async fn get_user_conversations(pool: &MySqlPool, user_id: i64) -> Result<Vec<ConversationRes>, AppError> {
    sqlx::query_as(
      "SELECT c.* FROM conversations c
      JOIN conversation_members cm ON c.id = cm.conversation_id
      WHERE cm.user_id = ?"
    )
    .bind(&user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
  }

  pub async fn add_member(pool: &MySqlPool, user_id: i64, conversation_id: i64, role: MemberRole) -> Result<(), AppError> {
    sqlx::query(
      "INSERT INTO conversation_member (conversation_id, user_id, role) VALUES (?, ?, ?)"
    )
    .bind(&conversation_id)
    .bind(&user_id)
    .bind(role)
    .execute(pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(())
  }
}
