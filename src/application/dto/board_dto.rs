use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::domain::repositories::Board;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBoardDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Board name must be between 1 and 100 characters long"
    ))]
    pub name: String,
    #[validate(length(
        max = 1000,
        message = "Board description must be at most 1000 characters long"
    ))]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBoardDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Board name must be between 1 and 100 characters long"
    ))]
    pub name: Option<String>,
    #[validate(length(
        max = 1000,
        message = "Board description must be at most 1000 characters long"
    ))]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl BoardDto {
    pub fn from_domain(board: Board) -> Self {
        Self {
            id: board.id,
            name: board.name,
            description: board.description,
            owner_id: board.owner_id,
            created_at: board.created_at,
            updated_at: board.updated_at,
        }
    }
}
