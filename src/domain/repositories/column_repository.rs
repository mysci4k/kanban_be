use crate::shared::error::ApplicationError;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Column {
    pub id: Uuid,
    pub name: String,
    pub position: String,
    pub board_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl Column {
    pub fn new(id: Uuid, name: String, position: String, board_id: Uuid) -> Self {
        let now = Utc::now().fixed_offset();

        Self {
            id,
            name,
            position,
            board_id,
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
pub trait ColumnRepository: Send + Sync {
    async fn create(&self, column: Column) -> Result<Column, ApplicationError>;
    async fn find_by_id(&self, column_id: Uuid) -> Result<Option<Column>, ApplicationError>;
    async fn find_by_board_id(&self, board_id: Uuid) -> Result<Vec<Column>, ApplicationError>;
    async fn update(&self, column: Column) -> Result<Column, ApplicationError>;
    async fn delete(&self, column_id: Uuid) -> Result<u64, ApplicationError>;
}
