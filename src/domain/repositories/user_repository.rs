use crate::shared::error::ApplicationError;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl User {
    pub fn new(
        id: Uuid,
        email: String,
        password: String,
        first_name: String,
        last_name: String,
    ) -> Self {
        let now = Utc::now().fixed_offset();

        Self {
            id,
            email,
            password,
            first_name,
            last_name,
            is_active: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User) -> Result<User, ApplicationError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, ApplicationError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, ApplicationError>;
    async fn exists_by_id(&self, id: Uuid) -> Result<bool, ApplicationError>;
    async fn exists_by_email(&self, email: &str) -> Result<bool, ApplicationError>;
    async fn activate(&self, id: Uuid) -> Result<User, ApplicationError>;
    async fn update_password(&self, id: Uuid, new_password: &str)
    -> Result<User, ApplicationError>;
}
