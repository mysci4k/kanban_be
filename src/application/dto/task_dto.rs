use crate::domain::repositories::Task;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskDto {
    #[validate(length(
        min = 1,
        max = 254,
        message = "Task title must be between 1 and 254 characters long"
    ))]
    pub title: String,
    pub description: Option<String>,
    #[validate(custom(
        function = validate_tags,
        message = "Each tag must be between 1 and 50 characters long"
    ))]
    pub tags: Option<Vec<String>>,
    pub column_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskDto {
    #[validate(length(
        min = 1,
        max = 254,
        message = "Task title must be between 1 and 254 characters long"
    ))]
    pub title: Option<String>,
    pub description: Option<String>,
    #[validate(custom(
        function = validate_tags,
        message = "Each tag must be between 1 and 50 characters long"
    ))]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub position: String,
    pub column_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl TaskDto {
    pub fn from_domain(task: Task) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            tags: task.tags,
            position: task.position,
            column_id: task.column_id,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }
}

fn validate_tags(tags: &[String]) -> Result<(), ValidationError> {
    for tag in tags {
        if tag.is_empty() || tag.len() > 50 {
            return Err(ValidationError::new("tags"));
        }
    }

    Ok(())
}
