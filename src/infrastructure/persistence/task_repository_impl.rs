use crate::{
    domain::repositories::{Task, TaskRepository},
    shared::error::ApplicationError,
};
use async_trait::async_trait;
use entity::{TaskActiveModel, TaskColumn, TaskEntity, TaskModel};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::{debug, error, instrument};
use uuid::Uuid;

pub struct SeaOrmTaskRepository {
    db: DatabaseConnection,
}

impl SeaOrmTaskRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn to_domain(model: TaskModel) -> Task {
        Task {
            id: model.id,
            title: model.title,
            description: model.description,
            tags: model.tags,
            position: model.position,
            column_id: model.column_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    fn to_active_model(task: Task) -> TaskActiveModel {
        TaskActiveModel {
            id: Set(task.id),
            title: Set(task.title),
            description: Set(task.description),
            tags: Set(task.tags),
            position: Set(task.position),
            column_id: Set(task.column_id),
            created_at: Set(task.created_at),
            updated_at: Set(task.updated_at),
        }
    }
}

#[async_trait]
impl TaskRepository for SeaOrmTaskRepository {
    #[instrument(
        name = "db.task.create",
        skip(self, task),
        fields(
            db.operation = "insert",
            task.id = %task.id
        ),
        err
    )]
    async fn create(&self, task: Task) -> Result<Task, ApplicationError> {
        debug!("Creating task");
        let active_model = Self::to_active_model(task);

        let result = TaskEntity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to create task");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(Self::to_domain(result))
    }

    #[instrument(
        name = "db.task.find_by_id",
        skip(self, task_id),
        fields(
            db.operation = "select",
            task.id = %task_id
        ),
        err
    )]
    async fn find_by_id(&self, task_id: Uuid) -> Result<Option<Task>, ApplicationError> {
        debug!("Finding task by ID");
        let result = TaskEntity::find_by_id(task_id)
            .one(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to find task by ID");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.map(Self::to_domain))
    }

    #[instrument(
        name = "db.task.find_by_column_id",
        skip(self, column_id),
        fields(
            db.operation = "select",
            column.id = %column_id
        ),
        err
    )]
    async fn find_by_column_id(&self, column_id: Uuid) -> Result<Vec<Task>, ApplicationError> {
        debug!("Finding tasks by column ID");
        let result = TaskEntity::find()
            .filter(TaskColumn::ColumnId.eq(column_id))
            .all(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to find tasks by column ID");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.into_iter().map(Self::to_domain).collect())
    }

    #[instrument(
        name = "db.task.update",
        skip(self, task),
        fields(
            db.operation = "update",
            task.id = %task.id
        ),
        err
    )]
    async fn update(&self, task: Task) -> Result<Task, ApplicationError> {
        debug!("Updating task");
        let active_model = Self::to_active_model(task);

        let result = TaskEntity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to update task");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(Self::to_domain(result))
    }

    #[instrument(
        name = "db.task.delete",
        skip(self, task_id),
        fields(
            db.operation = "delete",
            task.id = %task_id
        ),
        err
    )]
    async fn delete(&self, task_id: Uuid) -> Result<u64, ApplicationError> {
        debug!("Deleting task");
        let result = TaskEntity::delete_by_id(task_id)
            .exec(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to delete task");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.rows_affected)
    }
}
