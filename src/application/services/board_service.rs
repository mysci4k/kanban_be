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
use tracing::{info, instrument, warn};
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

    #[instrument(
        name = "board.create_board",
        skip(self, dto, owner_id),
        fields(user.id = %owner_id),
        err
    )]
    pub async fn create_board(
        &self,
        dto: CreateBoardDto,
        owner_id: Uuid,
    ) -> Result<BoardDto, ApplicationError> {
        dto.validate()?;

        let board_id = Uuid::now_v7();
        let board = Board::new(board_id, dto.name, dto.description, owner_id);

        let board_member = BoardMember::new(
            Uuid::now_v7(),
            board_id,
            owner_id,
            BoardMemberRoleEnum::Owner,
        );

        let saved_board = self
            .board_repository
            .create_with_member(board, board_member)
            .await?;

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

        info!(board.id = %board_id, "Board created successfully");
        Ok(BoardDto::from_domain(saved_board))
    }

    #[instrument(
        name = "board.get_board_by_id",
        skip(self, board_id, user_id),
        fields(
           board.id = %board_id,
           user.id = %user_id
        ),
        err
    )]
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

        info!(board.id = %board_id, "Board retrieved successfully");
        Ok(BoardDto::from_domain(board))
    }

    #[instrument(
        name = "board.get_boards_by_membership",
        skip(self, user_id),
        fields(user.id = %user_id),
        err
    )]
    pub async fn get_boards_by_membership(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<BoardDto>, ApplicationError> {
        let boards = self.board_repository.find_by_membership(user_id).await?;

        info!(board.count = %boards.len(), "Boards retrieved successfully");
        Ok(boards.into_iter().map(BoardDto::from_domain).collect())
    }

    #[instrument(
        name = "board.update_board",
        skip(self, dto, board_id, user_id),
        fields(
           board.id = %board_id,
           user.id = %user_id
        ),
        err
    )]
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
            warn!(
                board.id = %board_id,
                user.id = %user_id,
                "Attempt to update board without sufficient permissions"
            );
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

        info!(board.id = %board_id, "Board updated successfully");
        Ok(BoardDto::from_domain(updated_board))
    }

    #[instrument(
        name = "board.delete_board",
        skip(self, board_id, user_id),
        fields(
           board.id = %board_id,
           user.id = %user_id
        ),
        err
    )]
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
            warn!(
                board.id = %board_id,
                user.id = %user_id,
                "Attempt to delete board without sufficient permissions"
            );
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

        info!(board.id = %board_id, "Board deleted successfully");
        Ok(deleted_board)
    }

    #[instrument(
        name = "board.add_board_member",
        skip(self, dto, user_id),
        fields(
           board.id = %dto.board_id,
           new_member.id = %dto.user_id,
           user.id = %user_id
        ),
        err
    )]
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
            warn!(
                board.id = %dto.board_id,
                user.id = %user_id,
                "Attempt to add board member without sufficient permissions"
            );
            return Err(ApplicationError::Forbidden {
                message: "You don't have permission to perform this action".to_string(),
            });
        }

        if !self.user_repository.exists_by_id(dto.user_id).await? {
            warn!(
                new_member.id = %dto.user_id,
                "Attempt to add non-existent user as board member"
            );
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

        info!(
            board.id = %dto.board_id,
            new_member.id = %dto.user_id,
            "Board member added successfully"
        );
        Ok(BoardMemberDto::from_domain(saved_board_member))
    }

    #[instrument(
        name = "board.get_board_members",
        skip(self, board_id, user_id),
        fields(
           board.id = %board_id,
           user.id = %user_id
        ),
        err
    )]
    pub async fn get_board_members(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<BoardMemberDto>, ApplicationError> {
        if self
            .board_member_repository
            .find_by_board_and_user_id(board_id, user_id)
            .await?
            .is_none()
        {
            warn!(
                board.id = %board_id,
                user.id = %user_id,
                "Attempt to access board members without access to the board"
            );
            return Err(ApplicationError::Forbidden {
                message: "You don't have access to this board".to_string(),
            });
        }

        let board_members = self
            .board_member_repository
            .find_by_board_id(board_id)
            .await?;

        info!(board.id = %board_id, "Board members retrieved successfully");
        Ok(board_members
            .into_iter()
            .map(BoardMemberDto::from_domain)
            .collect())
    }

    #[instrument(
        name = "board.update_board_member_role",
        skip(self, dto, user_id),
        fields(
           board.id = %dto.board_id,
           target_member.id = %dto.user_id,
           user.id = %user_id
        ),
        err
    )]
    pub async fn update_board_member_role(
        &self,
        dto: UpdateBoardMemberRoleDto,
        user_id: Uuid,
    ) -> Result<BoardMemberDto, ApplicationError> {
        dto.validate()?;

        if dto.user_id == user_id {
            warn!(
                target_member.id = %dto.user_id,
                user.id = %user_id,
                "Attempt to change user's own role"
            );
            return Err(ApplicationError::Conflict {
                message: "You cannot change your own role".to_string(),
            });
        }

        if dto.role == BoardMemberRoleEnum::Owner {
            warn!(
                target_member.id = %dto.user_id,
                user.id = %user_id,
                "Attempt to assign Owner role to another user"
            );
            return Err(ApplicationError::Conflict {
                message: "You cannot assign the Owner role to another user".to_string(),
            });
        }

        if !self
            .board_member_repository
            .check_permissions(dto.board_id, user_id, vec![BoardMemberRoleEnum::Owner])
            .await?
        {
            warn!(
                board.id = %dto.board_id,
                user.id = %user_id,
                "Attempt to change board member role without sufficient permissions"
            );
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

        info!(
            board.id = %dto.board_id,
            target_member.id = %dto.user_id,
            "Board member role updated successfully"
        );
        Ok(BoardMemberDto::from_domain(updated_board_member))
    }

    #[instrument(
        name = "board.delete_board_member",
        skip(self, dto, user_id),
        fields(
            board.id = %dto.board_id,
            target_member.id = %dto.user_id,
            user.id = %user_id
        ),
        err
    )]
    pub async fn delete_board_member(
        &self,
        dto: DeleteBoardMemberDto,
        user_id: Uuid,
    ) -> Result<u64, ApplicationError> {
        dto.validate()?;

        if dto.user_id == user_id {
            warn!(
                target_member.id = %dto.user_id,
                user.id = %user_id,
                "Attempt to remove oneself from board"
            );
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
            warn!(
                board.id = %dto.board_id,
                user.id = %user_id,
                "Attempt to remove board member without sufficient permissions"
            );
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
            warn!(
                board.id = %dto.board_id,
                target_member.id = %dto.user_id,
                user.id = %user_id,
                "Attempt to remove board member with equal or higher role"
            );
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

        info!(
            board.id = %dto.board_id,
            target_member.id = %dto.user_id,
            "Board member removed successfully"
        );
        Ok(deleted_board_member)
    }
}
