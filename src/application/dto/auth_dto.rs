use crate::shared::utils::constants::RE_SPECIAL_CHARS;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivationQueryDto {
    #[validate(length(min = 1, message = "User ID is required"))]
    pub user_id: String,
    #[validate(length(min = 1, message = "Activation token is required"))]
    pub activation_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResendActivationQueryDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ForgotPasswordQueryDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordDto {
    #[validate(length(min = 1, message = "User ID is required"))]
    pub user_id: String,
    #[validate(length(min = 1, message = "Reset token is required"))]
    pub reset_token: String,
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
    pub new_password: String,
}
