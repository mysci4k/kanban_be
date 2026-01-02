use crate::{
    application::dto::UserDto, domain::repositories::UserRepository,
    shared::error::ApplicationError,
};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

pub struct UserService {
    user_repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    #[instrument(
        name = "user.get_user_by_id",
        skip(self, user_id),
        fields(user.id = %user_id),
        err
    )]
    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<UserDto, ApplicationError> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "User with the given ID not found".to_string(),
            })?;

        info!(user.id = %user.id, "User retrieved successfully");
        Ok(UserDto::from_domain(user))
    }
}
