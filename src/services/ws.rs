use sqlx::{MySqlPool};
use crate::{error::AppError, handlers::ClientMessage};

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ChatMessage {
    pub sender_id: i64,
    pub content: String,
    pub send_time: chrono::DateTime<chrono::Utc>
}

pub struct MessageRepository;

impl MessageRepository {
    // TODO(human): 实现 save 方法，将聊天消息插入数据库
    // 参数：pool, sender_id, conversation_id, content
    // SQL: INSERT INTO messages (conversation_id, sender_id, content, msg_type) VALUES (?, ?, ?, 'text')
    pub async fn save(pool: &MySqlPool, sender_id: i64, conversation_id: i64, content: &str) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO messages (conversation_id, sender_id, content, msg_type) VALUES (?, ?, ?, 'text')"
        )
        .bind(&conversation_id)
        .bind(&sender_id)
        .bind(content)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn get_recent(pool: &MySqlPool, conversation_id: i64, limit: i16 ) -> Result<Vec<ChatMessage>, AppError> {
        sqlx::query_as(
            "SELECT sender_id, content, created_at FROM messages WHERE conversation_id = ? ORDER BY created_at DESC LIMIT ?"
        )
        .bind(&conversation_id)
        .bind(&limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
    }
}
