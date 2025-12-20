use crate::{
    application::dto::{CreateTaskDto, TaskDto, UpdateTaskDto},
    domain::{
        events::{
            BoardEvent, SharedEventBus, TaskCreatedEvent, TaskDeletedEvent, TaskMovedEvent,
            TaskUpdatedEvent,
        },
        repositories::{BoardMemberRepository, ColumnRepository, Task, TaskRepository},
    },
    shared::{error::ApplicationError, utils::FractionalIndexGenerator},
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub struct TaskService {
    task_repository: Arc<dyn TaskRepository>,
    column_repository: Arc<dyn ColumnRepository>,
    board_member_repository: Arc<dyn BoardMemberRepository>,
    event_bus: SharedEventBus,
}

impl TaskService {
    pub fn new(
        task_repository: Arc<dyn TaskRepository>,
        column_repository: Arc<dyn ColumnRepository>,
        board_member_repository: Arc<dyn BoardMemberRepository>,
        event_bus: SharedEventBus,
    ) -> Self {
        Self {
            task_repository,
            column_repository,
            board_member_repository,
            event_bus,
        }
    }

    pub async fn create_task(
        &self,
        dto: CreateTaskDto,
        user_id: Uuid,
    ) -> Result<TaskDto, ApplicationError> {
        dto.validate()?;

        let column = self
            .column_repository
            .find_by_id(dto.column_id)
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

        let mut existing_tasks = self
            .task_repository
            .find_by_column_id(dto.column_id)
            .await?;

        existing_tasks.sort_by(|a, b| a.position.cmp(&b.position));
        let existing_positions: Vec<String> =
            existing_tasks.iter().map(|t| t.position.clone()).collect();

        let position = if existing_positions.is_empty() {
            FractionalIndexGenerator::first()
        } else {
            FractionalIndexGenerator::after(&existing_positions[existing_positions.len() - 1])
                .map_err(|err| ApplicationError::BadRequest {
                    message: format!("Failed to generate position: {}", err),
                })?
        };

        let task = Task::new(
            Uuid::now_v7(),
            dto.title,
            dto.description,
            dto.tags,
            position,
            dto.column_id,
        );

        let saved_task = self.task_repository.create(task).await?;

        self.event_bus
            .publish(
                column.board_id,
                BoardEvent::TaskCreated(TaskCreatedEvent {
                    task_id: saved_task.id,
                    title: saved_task.title.clone(),
                    description: saved_task.description.clone(),
                    tags: saved_task.tags.clone(),
                    position: saved_task.position.clone(),
                    column_id: saved_task.column_id,
                    created_by: user_id,
                    timestamp: saved_task.created_at,
                }),
            )
            .await;

        Ok(TaskDto::from_domain(saved_task))
    }

    pub async fn get_task_by_id(
        &self,
        task_id: Uuid,
        user_id: Uuid,
    ) -> Result<TaskDto, ApplicationError> {
        let task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Task with the given ID not found".to_string(),
            })?;

        let column = self
            .column_repository
            .find_by_id(task.column_id)
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

        Ok(TaskDto::from_domain(task))
    }

    pub async fn get_column_tasks(
        &self,
        column_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<TaskDto>, ApplicationError> {
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

        let mut tasks = self.task_repository.find_by_column_id(column_id).await?;

        tasks.sort_by(|a, b| a.position.cmp(&b.position));

        Ok(tasks.into_iter().map(TaskDto::from_domain).collect())
    }

    pub async fn update_task(
        &self,
        dto: UpdateTaskDto,
        task_id: Uuid,
        user_id: Uuid,
    ) -> Result<TaskDto, ApplicationError> {
        dto.validate()?;

        let mut task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Task with the given ID not found".to_string(),
            })?;

        let column = self
            .column_repository
            .find_by_id(task.column_id)
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

        if let Some(title) = dto.title {
            task.title = title;
        }
        if dto.description.is_some() {
            task.description = dto.description;
        }
        if dto.tags.is_some() {
            task.tags = dto.tags;
        }
        task.updated_at = Utc::now().fixed_offset();

        let updated_task = self.task_repository.update(task).await?;

        self.event_bus
            .publish(
                column.board_id,
                BoardEvent::TaskUpdated(TaskUpdatedEvent {
                    task_id,
                    title: Some(updated_task.title.clone()),
                    description: updated_task.description.clone(),
                    tags: updated_task.tags.clone(),
                    updated_by: user_id,
                    timestamp: updated_task.updated_at,
                }),
            )
            .await;

        Ok(TaskDto::from_domain(updated_task))
    }

    pub async fn move_task(
        &self,
        target_position: usize,
        task_id: Uuid,
        column_id: Uuid,
        user_id: Uuid,
    ) -> Result<TaskDto, ApplicationError> {
        let task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Task with the given ID not found".to_string(),
            })?;

        let old_column = self
            .column_repository
            .find_by_id(task.column_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Source column with the given ID not found".to_string(),
            })?;

        let new_column = self
            .column_repository
            .find_by_id(column_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Target column with the given ID not found".to_string(),
            })?;

        if old_column.board_id != new_column.board_id {
            return Err(ApplicationError::BadRequest {
                message: "Cannot move task between columns of different boards".to_string(),
            });
        }

        if self
            .board_member_repository
            .find_by_board_and_user_id(old_column.board_id, user_id)
            .await?
            .is_none()
        {
            return Err(ApplicationError::Forbidden {
                message: "You don't have access to this board".to_string(),
            });
        }

        let mut target_column_tasks = self.task_repository.find_by_column_id(column_id).await?;

        target_column_tasks.sort_by(|a, b| a.position.cmp(&b.position));

        let other_tasks: Vec<String> = target_column_tasks
            .iter()
            .filter(|t| t.id != task_id)
            .map(|t| t.position.clone())
            .collect();

        if target_position > other_tasks.len() {
            return Err(ApplicationError::BadRequest {
                message: format!(
                    "Target position is out of bounds (0 - {})",
                    other_tasks.len()
                ),
            });
        }

        let new_position =
            FractionalIndexGenerator::generate_for_position(&other_tasks, target_position)
                .map_err(|err| ApplicationError::BadRequest {
                    message: format!("Failed to generate position: {}", err),
                })?;

        let old_position = if task.column_id == column_id {
            target_column_tasks
                .iter()
                .position(|t| t.id == task_id)
                .unwrap_or(0)
        } else {
            0
        };

        let mut updated_task = task.clone();
        updated_task.column_id = column_id;
        updated_task.position = new_position;
        updated_task.updated_at = Utc::now().fixed_offset();

        let saved_task = self.task_repository.update(updated_task).await?;

        self.event_bus
            .publish(
                old_column.board_id,
                BoardEvent::TaskMoved(TaskMovedEvent {
                    task_id,
                    old_column_id: task.column_id,
                    new_column_id: column_id,
                    old_position,
                    new_position: target_position,
                    moved_by: user_id,
                    timestamp: saved_task.updated_at,
                }),
            )
            .await;

        Ok(TaskDto::from_domain(saved_task))
    }

    pub async fn delete_task(&self, task_id: Uuid, user_id: Uuid) -> Result<u64, ApplicationError> {
        let task = self
            .task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| ApplicationError::NotFound {
                message: "Task with the given ID not found".to_string(),
            })?;

        let column = self
            .column_repository
            .find_by_id(task.column_id)
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

        let deleted_column = self.task_repository.delete(task_id).await?;

        self.event_bus
            .publish(
                column.board_id,
                BoardEvent::TaskDeleted(TaskDeletedEvent {
                    task_id,
                    deleted_by: user_id,
                    timestamp: Utc::now().fixed_offset(),
                }),
            )
            .await;

        Ok(deleted_column)
    }
}
