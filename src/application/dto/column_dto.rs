use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domain::repositories::Column;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateColumnDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Column name must be between 1 and 100 characters long"
    ))]
    pub name: String,
    pub board_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateColumnDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Column name must be between 1 and 100 characters long"
    ))]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDto {
    pub id: Uuid,
    pub name: String,
    pub position: String,
    pub board_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl ColumnDto {
    pub fn from_domain(column: Column) -> Self {
        Self {
            id: column.id,
            name: column.name,
            position: column.position,
            board_id: column.board_id,
            created_at: column.created_at,
            updated_at: column.updated_at,
        }
    }
}
