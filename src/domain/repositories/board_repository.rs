use crate::shared::error::ApplicationError;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl Board {
    pub fn new(id: Uuid, name: String, description: Option<String>, owner_id: Uuid) -> Self {
        let now = Utc::now().fixed_offset();

        Self {
            id,
            name,
            description,
            owner_id,
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
pub trait BoardRepository: Send + Sync {
    async fn create(&self, board: Board) -> Result<Board, ApplicationError>;
    async fn find_by_id(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Board>, ApplicationError>;
    async fn find_by_membership(&self, user_id: Uuid) -> Result<Vec<Board>, ApplicationError>;
    async fn update(&self, board: Board) -> Result<Board, ApplicationError>;
    async fn delete(&self, board_id: Uuid) -> Result<u64, ApplicationError>;
}
