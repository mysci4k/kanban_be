use crate::shared::error::ApplicationError;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub position: String,
    pub column_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl Task {
    pub fn new(
        id: Uuid,
        title: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
        position: String,
        column_id: Uuid,
    ) -> Self {
        let now = Utc::now().fixed_offset();

        Self {
            id,
            title,
            description,
            tags,
            position,
            column_id,
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn create(&self, task: Task) -> Result<Task, ApplicationError>;
    async fn find_by_id(&self, task_id: Uuid) -> Result<Option<Task>, ApplicationError>;
    async fn find_by_column_id(&self, column_id: Uuid) -> Result<Vec<Task>, ApplicationError>;
    async fn update(&self, task: Task) -> Result<Task, ApplicationError>;
    async fn delete(&self, task_id: Uuid) -> Result<u64, ApplicationError>;
}
