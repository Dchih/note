use sqlx::MySqlPool;
use crate::error::AppError;

pub struct MessageRepository;

impl MessageRepository {
    // TODO(human): 实现 save 方法，将聊天消息插入数据库
    // 参数：pool, sender_id, conversation_id, content
    // SQL: INSERT INTO messages (conversation_id, sender_id, content, msg_type) VALUES (?, ?, ?, 'text')
}
