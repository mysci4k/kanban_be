use crate::{
    application::dto::{ColumnDto, CreateColumnDto, UpdateColumnDto},
    domain::{
        events::{
            BoardEvent, ColumnCreatedEvent, ColumnDeletedEvent, ColumnMovedEvent,
            ColumnUpdatedEvent, SharedEventBus,
        },
        repositories::{BoardMemberRepository, Column, ColumnRepository},
    },
    shared::{error::ApplicationError, utils::FractionalIndexGenerator},
};
use chrono::Utc;
use entity::BoardMemberRoleEnum;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub struct ColumnService {
    column_repository: Arc<dyn ColumnRepository>,
    board_member_repository: Arc<dyn BoardMemberRepository>,
    event_bus: SharedEventBus,
}

impl ColumnService {
    pub fn new(
        column_repository: Arc<dyn ColumnRepository>,
        board_member_repository: Arc<dyn BoardMemberRepository>,
        event_bus: SharedEventBus,
    ) -> Self {
        Self {
            column_repository,
            board_member_repository,
            event_bus,
        }
    }

    pub async fn create_column(
        &self,
        dto: CreateColumnDto,
        user_id: Uuid,
    ) -> Result<ColumnDto, ApplicationError> {
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

        let mut existing_columns = self
            .column_repository
            .find_by_board_id(dto.board_id)
            .await?;

        existing_columns.sort_by(|a, b| a.position.cmp(&b.position));
        let existing_positions: Vec<String> = existing_columns
            .iter()
            .map(|c| c.position.clone())
            .collect();

        let position = if existing_positions.is_empty() {
            FractionalIndexGenerator::first()
        } else {
            FractionalIndexGenerator::after(&existing_positions[existing_positions.len() - 1])
                .map_err(|err| ApplicationError::BadRequest {
                    message: format!("Failed to generate position: {}", err),
                })?
        };

        let column = Column::new(Uuid::now_v7(), dto.name, position, dto.board_id);

        let saved_column = self.column_repository.create(column).await?;

        self.event_bus
            .publish(
                dto.board_id,
                BoardEvent::ColumnCreated(ColumnCreatedEvent {
                    column_id: saved_column.id,
                    name: saved_column.name.clone(),
                    position: saved_column.position.clone(),
                    board_id: saved_column.board_id,
                    created_by: user_id,
                    timestamp: saved_column.created_at,
                }),
            )
            .await;

        Ok(ColumnDto::from_domain(saved_column))
    }

    pub async fn get_column_by_id(
        &self,
        column_id: Uuid,
        user_id: Uuid,
    ) -> Result<ColumnDto, ApplicationError> {
        let column = self
            .column_repository
            .find_by_id(column_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Column with the given ID not found".to_string(),
            })?;

        if self
            .board_member_repository
            .find_by_board_and_user_id(column.board_id, user_id)
            .await?
            .is_none()
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have access to this board".to_string(),
            });
        }

        Ok(ColumnDto::from_domain(column))
    }

    pub async fn get_board_columns(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<ColumnDto>, ApplicationError> {
        if self
            .board_member_repository
            .find_by_board_and_user_id(board_id, user_id)
            .await?
            .is_none()
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have access to this board".to_string(),
            });
        }

        let mut columns = self.column_repository.find_by_board_id(board_id).await?;

        columns.sort_by(|a, b| a.position.cmp(&b.position));

        Ok(columns.into_iter().map(ColumnDto::from_domain).collect())
    }

    pub async fn update_column(
        &self,
        dto: UpdateColumnDto,
        column_id: Uuid,
        user_id: Uuid,
    ) -> Result<ColumnDto, ApplicationError> {
        dto.validate()?;

        let mut column = self
            .column_repository
            .find_by_id(column_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Column with the given ID not found".to_string(),
            })?;

        if !self
            .board_member_repository
            .check_permissions(
                column.board_id,
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
            column.name = name;
        }
        column.updated_at = Utc::now().fixed_offset();

        let updated_column = self.column_repository.update(column).await?;

        self.event_bus
            .publish(
                updated_column.board_id,
                BoardEvent::ColumnUpdated(ColumnUpdatedEvent {
                    column_id,
                    name: Some(updated_column.name.clone()),
                    updated_by: user_id,
                    timestamp: updated_column.updated_at,
                }),
            )
            .await;

        Ok(ColumnDto::from_domain(updated_column))
    }

    pub async fn move_column(
        &self,
        target_position: usize,
        column_id: Uuid,
        user_id: Uuid,
    ) -> Result<ColumnDto, ApplicationError> {
        let column = self
            .column_repository
            .find_by_id(column_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Column with the given ID not found".to_string(),
            })?;

        if !self
            .board_member_repository
            .check_permissions(
                column.board_id,
                user_id,
                vec![BoardMemberRoleEnum::Owner, BoardMemberRoleEnum::Moderator],
            )
            .await?
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have permission to perform this action".to_string(),
            });
        }

        let mut all_columns = self
            .column_repository
            .find_by_board_id(column.board_id)
            .await?;

        all_columns.sort_by(|a, b| a.position.cmp(&b.position));

        let current_index = all_columns
            .iter()
            .position(|c| c.id == column.id)
            .ok_or_else(|| ApplicationError::InternalError {
                message: "Failed to determine current column index".to_string(),
            })?;

        if current_index == target_position {
            return Ok(ColumnDto::from_domain(column));
        }

        if target_position >= all_columns.len() {
            return Err(ApplicationError::BadRequest {
                message: format!(
                    "Target position is out of bounds (0 - {})",
                    all_columns.len() - 1
                ),
            });
        }

        let other_columns: Vec<String> = all_columns
            .iter()
            .filter(|c| c.id != column_id)
            .map(|c| c.position.clone())
            .collect();

        let new_position =
            FractionalIndexGenerator::generate_for_position(&other_columns, target_position)
                .map_err(|err| ApplicationError::BadRequest {
                    message: format!("Failed to calculate new position: {}", err),
                })?;

        let mut updated_column = column.clone();
        updated_column.position = new_position;
        updated_column.updated_at = Utc::now().fixed_offset();

        let saved_column = self.column_repository.update(updated_column).await?;

        self.event_bus
            .publish(
                saved_column.board_id,
                BoardEvent::ColumnMoved(ColumnMovedEvent {
                    column_id,
                    old_position: current_index,
                    new_position: target_position,
                    moved_by: user_id,
                    timestamp: saved_column.updated_at,
                }),
            )
            .await;

        Ok(ColumnDto::from_domain(saved_column))
    }

    pub async fn delete_column(
        &self,
        column_id: Uuid,
        user_id: Uuid,
    ) -> Result<u64, ApplicationError> {
        let column = self
            .column_repository
            .find_by_id(column_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Column with the given ID not found".to_string(),
            })?;

        if !self
            .board_member_repository
            .check_permissions(
                column.board_id,
                user_id,
                vec![BoardMemberRoleEnum::Owner, BoardMemberRoleEnum::Moderator],
            )
            .await?
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have permission to perform this action".to_string(),
            });
        }

        let deleted_column = self.column_repository.delete(column_id).await?;

        self.event_bus
            .publish(
                column.board_id,
                BoardEvent::ColumnDeleted(ColumnDeletedEvent {
                    column_id,
                    deleted_by: user_id,
                    timestamp: Utc::now().fixed_offset(),
                }),
            )
            .await;

        Ok(deleted_column)
    }
}
