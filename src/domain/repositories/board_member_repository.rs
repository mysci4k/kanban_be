use crate::shared::error::ApplicationError;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use entity::BoardMemberRoleEnum;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BoardMember {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub role: BoardMemberRoleEnum,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl BoardMember {
    pub fn new(id: Uuid, board_id: Uuid, user_id: Uuid, role: BoardMemberRoleEnum) -> Self {
        let now = Utc::now().fixed_offset();

        Self {
            id,
            board_id,
            user_id,
            role,
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
pub trait BoardMemberRepository: Send + Sync {
    async fn create(&self, board_member: BoardMember) -> Result<BoardMember, ApplicationError>;
    async fn find_by_board_and_user_id(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BoardMember>, ApplicationError>;
    async fn get_role(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BoardMemberRoleEnum>, ApplicationError>;
    async fn check_permissions(
        &self,
        board_id: Uuid,
        user_id: Uuid,
        member_roles: Vec<BoardMemberRoleEnum>,
    ) -> Result<bool, ApplicationError>;
    async fn update(&self, board_member: BoardMember) -> Result<BoardMember, ApplicationError>;
    async fn delete(&self, board_id: Uuid, user_id: Uuid) -> Result<u64, ApplicationError>;
}
