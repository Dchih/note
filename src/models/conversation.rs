use serde::Serialize;
use sqlx::FromRow;
use chrono::{ DateTime, Utc };

#[derive(Debug, Clone, sqlx::Type, Serialize)]
#[sqlx(type_name = "SMALLINT")]
#[repr(i16)]
pub enum ConversationType {
  Private = 0,
  Group = 1,
}

#[derive(Debug, FromRow)]
pub struct _Conversation {
  pub id: i64,
  #[sqlx(rename = "type")]
  pub conversation_type: ConversationType,
  pub name: Option<String>,
  pub created_at: DateTime<Utc>,
  pub is_deleted: bool
}


#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum MemberRole {
  Owner,
  Admin,
  Member
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct ConversationMember {
  pub id: i64,
  pub conversation_id: i64,
  pub user_id: i64,
  pub role: MemberRole,
  pub joined_at: DateTime<Utc>
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum _MessageType {
  Text,
  Image,
  File
}

#[derive(Debug, FromRow)]
pub struct _Message {
  pub id: i64,
  pub conversation_id: i64,
  pub sender_id: i64, 
  pub content: String,
  pub msg_type: _MessageType,
  pub created_at: DateTime<Utc>
}