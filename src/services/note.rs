use sqlx::MySqlPool;
use crate::error::AppError;
use crate::models::{Note, CreateNote, UpdateNote};

pub struct NoteService;

impl NoteService {
    pub async fn find_all(pool: &MySqlPool, user_id: i64) -> Result<Vec<Note>, AppError> {
        sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE user_id = ? ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Note, AppError> {
        sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Note {} not found", id)))
    }

    pub async fn create(pool: &MySqlPool, data: CreateNote, user_id: i64) -> Result<Note, AppError> {
        tracing::info!("Creating note: {:?}", data);
        let result = sqlx::query("INSERT INTO notes (title, content, user_id) VALUES (?, ?, ?)")
            .bind(&data.title)
            .bind(&data.content)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tracing::info!("Inserted, last_insert_id: {}", result.last_insert_id());
        Self::find_by_id(pool, result.last_insert_id() as i64).await
    }

    pub async fn update(pool: &MySqlPool, id: i64, data: UpdateNote) -> Result<Note, AppError> {
        // 先确认存在
        Self::find_by_id(pool, id).await?;

        sqlx::query("UPDATE notes SET title = COALESCE(?, title), content = COALESCE(?, content) WHERE id = ?")
            .bind(&data.title)
            .bind(&data.content)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Self::find_by_id(pool, id).await
    }

    pub async fn delete(pool: &MySqlPool, id: i64) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM notes WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Note {} not found", id)));
        }

        Ok(())
    }
}