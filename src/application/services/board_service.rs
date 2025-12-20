use crate::{
    application::dto::{
        AddBoardMemberDto, BoardDto, BoardMemberDto, CreateBoardDto, DeleteBoardMemberDto,
        UpdateBoardDto, UpdateBoardMemberRoleDto,
    },
    domain::{
        events::{
            BoardCreatedEvent, BoardDeletedEvent, BoardEvent, BoardUpdatedEvent, MemberAddedEvent,
            MemberRemovedEvent, MemberRoleChangedEvent, SharedEventBus,
        },
        repositories::{
            Board, BoardMember, BoardMemberRepository, BoardRepository, UserRepository,
        },
    },
    shared::error::ApplicationError,
};
use chrono::Utc;
use entity::BoardMemberRoleEnum;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub struct BoardService {
    user_repository: Arc<dyn UserRepository>,
    board_repository: Arc<dyn BoardRepository>,
    board_member_repository: Arc<dyn BoardMemberRepository>,
    event_bus: SharedEventBus,
}

impl BoardService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        board_repository: Arc<dyn BoardRepository>,
        board_member_repository: Arc<dyn BoardMemberRepository>,
        event_bus: SharedEventBus,
    ) -> Self {
        Self {
            user_repository,
            board_repository,
            board_member_repository,
            event_bus,
        }
    }

    pub async fn create_board(
        &self,
        dto: CreateBoardDto,
        owner_id: Uuid,
    ) -> Result<BoardDto, ApplicationError> {
        dto.validate()?;

        let board_id = Uuid::now_v7();
        let board = Board::new(board_id, dto.name, dto.description, owner_id);

        let saved_board = self.board_repository.create(board).await?;

        let board_member = BoardMember::new(
            Uuid::now_v7(),
            board_id,
            owner_id,
            BoardMemberRoleEnum::Owner,
        );

        self.board_member_repository.create(board_member).await?;

        self.event_bus
            .publish(
                board_id,
                BoardEvent::BoardCreated(BoardCreatedEvent {
                    board_id,
                    name: saved_board.name.clone(),
                    description: saved_board.description.clone(),
                    owner_id,
                    timestamp: saved_board.created_at,
                }),
            )
            .await;

        Ok(BoardDto::from_domain(saved_board))
    }

    pub async fn get_board_by_id(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<BoardDto, ApplicationError> {
        let board = self
            .board_repository
            .find_by_id(board_id, user_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Board with the given ID not found".to_string(),
            })?;

        Ok(BoardDto::from_domain(board))
    }

    pub async fn get_boards_by_membership(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<BoardDto>, ApplicationError> {
        let boards = self.board_repository.find_by_membership(user_id).await?;

        Ok(boards.into_iter().map(BoardDto::from_domain).collect())
    }

    pub async fn update_board(
        &self,
        dto: UpdateBoardDto,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<BoardDto, ApplicationError> {
        dto.validate()?;

        let mut board = self
            .board_repository
            .find_by_id(board_id, user_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Board with the given ID not found".to_string(),
            })?;

        if !self
            .board_member_repository
            .check_permissions(
                board_id,
                user_id,
                vec![BoardMemberRoleEnum::Owner, BoardMemberRoleEnum::Moderator],
            )
            .await?
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have permission to perform this action".to_string(),
            });
        }

        if let Some(name) = dto.name {
            board.name = name;
        }
        board.description = dto.description;
        board.updated_at = Utc::now().fixed_offset();

        let updated_board = self.board_repository.update(board).await?;

        self.event_bus
            .publish(
                board_id,
                BoardEvent::BoardUpdated(BoardUpdatedEvent {
                    board_id,
                    name: Some(updated_board.name.clone()),
                    description: updated_board.description.clone(),
                    updated_by: user_id,
                    timestamp: updated_board.updated_at,
                }),
            )
            .await;

        Ok(BoardDto::from_domain(updated_board))
    }

    pub async fn delete_board(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<u64, ApplicationError> {
        if !self
            .board_member_repository
            .check_permissions(board_id, user_id, vec![BoardMemberRoleEnum::Owner])
            .await?
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have permission to perform this action".to_string(),
            });
        }

        let deleted_board = self.board_repository.delete(board_id).await?;

        self.event_bus
            .publish(
                board_id,
                BoardEvent::BoardDeleted(BoardDeletedEvent {
                    board_id,
                    deleted_by: user_id,
                    timestamp: Utc::now().fixed_offset(),
                }),
            )
            .await;

        Ok(deleted_board)
    }

    pub async fn add_board_member(
        &self,
        dto: AddBoardMemberDto,
        user_id: Uuid,
    ) -> Result<BoardMemberDto, ApplicationError> {
        dto.validate()?;

        if !self
            .board_member_repository
            .check_permissions(
                dto.board_id,
                user_id,
                vec![BoardMemberRoleEnum::Owner, BoardMemberRoleEnum::Moderator],
            )
            .await?
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have permission to perform this action".to_string(),
            });
        }

        if !self.user_repository.exists_by_id(dto.user_id).await? {
            return Err(ApplicationError::NotFound {
                message: "User with the given ID not found".to_string(),
            });
        }

        let board_member = BoardMember::new(
            Uuid::now_v7(),
            dto.board_id,
            dto.user_id,
            BoardMemberRoleEnum::Member,
        );

        let saved_board_member = self.board_member_repository.create(board_member).await?;

        self.event_bus
            .publish(
                dto.board_id,
                BoardEvent::MemberAdded(MemberAddedEvent {
                    board_id: saved_board_member.board_id,
                    user_id: saved_board_member.user_id,
                    role: saved_board_member.role.clone(),
                    added_by: user_id,
                    timestamp: saved_board_member.created_at,
                }),
            )
            .await;

        Ok(BoardMemberDto::from_domain(saved_board_member))
    }

    pub async fn update_board_member_role(
        &self,
        dto: UpdateBoardMemberRoleDto,
        user_id: Uuid,
    ) -> Result<BoardMemberDto, ApplicationError> {
        dto.validate()?;

        if dto.user_id == user_id {
            return Err(ApplicationError::Conflict {
                message: "You cannot change your own role".to_string(),
            });
        }

        if dto.role == BoardMemberRoleEnum::Owner {
            return Err(ApplicationError::Conflict {
                message: "You cannot assign the Owner role to another user".to_string(),
            });
        }

        if !self
            .board_member_repository
            .check_permissions(dto.board_id, user_id, vec![BoardMemberRoleEnum::Owner])
            .await?
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have permission to perform this action".to_string(),
            });
        }

        let mut board_member = self
            .board_member_repository
            .find_by_board_and_user_id(dto.board_id, dto.user_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "The specified user is not a member of this board".to_string(),
            })?;

        board_member.role = dto.role;
        board_member.updated_at = Utc::now().fixed_offset();

        let updated_board_member = self.board_member_repository.update(board_member).await?;

        self.event_bus
            .publish(
                dto.board_id,
                BoardEvent::MemberRoleChanged(MemberRoleChangedEvent {
                    board_id: updated_board_member.board_id,
                    user_id: updated_board_member.user_id,
                    role: updated_board_member.role.clone(),
                    changed_by: user_id,
                    timestamp: updated_board_member.updated_at,
                }),
            )
            .await;

        Ok(BoardMemberDto::from_domain(updated_board_member))
    }

    pub async fn delete_board_member(
        &self,
        dto: DeleteBoardMemberDto,
        user_id: Uuid,
    ) -> Result<u64, ApplicationError> {
        dto.validate()?;

        if dto.user_id == user_id {
            return Err(ApplicationError::Conflict {
                message: "You cannot remove yourself from the board".to_string(),
            });
        }

        if !self
            .board_member_repository
            .check_permissions(
                dto.board_id,
                user_id,
                vec![BoardMemberRoleEnum::Owner, BoardMemberRoleEnum::Moderator],
            )
            .await?
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have permission to perform this action".to_string(),
            });
        }

        let requester_role = self
            .board_member_repository
            .get_role(dto.board_id, user_id)
            .await?
            .ok_or_else(|| ApplicationError::Forbidden {
                message: "You are not a member of this board".to_string(),
            })?;

        let target_role = self
            .board_member_repository
            .get_role(dto.board_id, dto.user_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "The specified user is not a member of this board".to_string(),
            })?;

        if requester_role.hierarchy_value() <= target_role.hierarchy_value() {
            return Err(ApplicationError::Forbidden {
                message: "You cannot remove a member with equal or higher role".to_string(),
            });
        }

        let deleted_board_member = self
            .board_member_repository
            .delete(dto.board_id, dto.user_id)
            .await?;

        self.event_bus
            .publish(
                dto.board_id,
                BoardEvent::MemberRemoved(MemberRemovedEvent {
                    board_id: dto.board_id,
                    user_id: dto.user_id,
                    removed_by: user_id,
                    timestamp: Utc::now().fixed_offset(),
                }),
            )
            .await;

        Ok(deleted_board_member)
    }
}
