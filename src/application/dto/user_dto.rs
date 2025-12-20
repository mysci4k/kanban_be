use crate::{
    domain::repositories::User,
    shared::utils::constants::{RE_ONLY_LETTERS, RE_SPECIAL_CHARS},
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserDto {
    #[validate(
        email(message = "Invalid email format"),
        length(
            min = 1,
            max = 254,
            message = "Email must be between 1 and 254 characters long"
        )
    )]
    pub email: String,
    #[validate(
        regex(
            path = RE_SPECIAL_CHARS,
            message = "Password must contain at least one special character"
        ),
        length(
            min = 8,
            max = 50,
            message = "Password must be between 8 and 50 characters long"
        )
    )]
    pub password: String,
    #[validate(
        regex(
            path = RE_ONLY_LETTERS,
            message = "First name must contain only letters"
        ),
        length(
            min = 1,
            max = 50,
            message = "First name must be between 1 and 50 characters long"
        )
    )]
    pub first_name: String,
    #[validate(
        regex(
            path = RE_ONLY_LETTERS,
            message = "Last name must contain only letters"
        ),
        length(
            min = 1,
            max = 50,
            message = "Last name must be between 1 and 50 characters long"
        )
    )]
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl UserDto {
    pub fn from_domain(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
