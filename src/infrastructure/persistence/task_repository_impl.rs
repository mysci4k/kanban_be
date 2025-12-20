use crate::{
    domain::repositories::{Task, TaskRepository},
    shared::error::ApplicationError,
};
use async_trait::async_trait;
use entity::{TaskActiveModel, TaskColumn, TaskEntity, TaskModel};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
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
    async fn create(&self, task: Task) -> Result<Task, ApplicationError> {
        let active_model = Self::to_active_model(task);

        let result = TaskEntity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(Self::to_domain(result))
    }

    async fn find_by_id(&self, task_id: Uuid) -> Result<Option<Task>, ApplicationError> {
        let result = TaskEntity::find_by_id(task_id)
            .one(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(result.map(Self::to_domain))
    }

    async fn find_by_column_id(&self, column_id: Uuid) -> Result<Vec<Task>, ApplicationError> {
        let result = TaskEntity::find()
            .filter(TaskColumn::ColumnId.eq(column_id))
            .all(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(result.into_iter().map(Self::to_domain).collect())
    }

    async fn update(&self, task: Task) -> Result<Task, ApplicationError> {
        let active_model = Self::to_active_model(task);

        let result = TaskEntity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(Self::to_domain(result))
    }

    async fn delete(&self, task_id: Uuid) -> Result<u64, ApplicationError> {
        let result = TaskEntity::delete_by_id(task_id)
            .exec(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(result.rows_affected)
    }
}
