use crate::{
    domain::repositories::{Column, ColumnRepository},
    shared::error::ApplicationError,
};
use async_trait::async_trait;
use entity::{ColumnActiveModel, ColumnColumn, ColumnEntity, ColumnModel};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use tracing::{debug, error, instrument};
use uuid::Uuid;

pub struct SeaOrmColumnRepository {
    db: DatabaseConnection,
}

impl SeaOrmColumnRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn to_domain(model: ColumnModel) -> Column {
        Column {
            id: model.id,
            name: model.name,
            position: model.position,
            board_id: model.board_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    fn to_active_model(column: Column) -> ColumnActiveModel {
        ColumnActiveModel {
            id: Set(column.id),
            name: Set(column.name),
            position: Set(column.position),
            board_id: Set(column.board_id),
            created_at: Set(column.created_at),
            updated_at: Set(column.updated_at),
        }
    }
}

#[async_trait]
impl ColumnRepository for SeaOrmColumnRepository {
    #[instrument(
        name = "db.column.create",
        skip(self, column),
        fields(
            db.operation = "insert",
            column.id = %column.id
        ),
        err
    )]
    async fn create(&self, column: Column) -> Result<Column, ApplicationError> {
        debug!("Creating column");
        let active_model = Self::to_active_model(column);

        let result = ColumnEntity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to create column");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(Self::to_domain(result))
    }

    #[instrument(
        name = "db.column.find_by_id",
        skip(self, column_id),
        fields(
            db.operation = "select",
            column.id = %column_id
        ),
        err
    )]
    async fn find_by_id(&self, column_id: Uuid) -> Result<Option<Column>, ApplicationError> {
        debug!("Finding column by ID");
        let result = ColumnEntity::find_by_id(column_id)
            .one(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to find column by ID");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.map(Self::to_domain))
    }

    #[instrument(
        name = "db.column.find_by_board_id",
        skip(self, board_id),
        fields(
            db.operation = "select",
            board.id = %board_id
        ),
        err
    )]
    async fn find_by_board_id(&self, board_id: Uuid) -> Result<Vec<Column>, ApplicationError> {
        debug!("Finding columns by board ID");
        let result = ColumnEntity::find()
            .filter(ColumnColumn::BoardId.eq(board_id))
            .order_by_asc(ColumnColumn::Position)
            .all(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to find columns by board ID");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.into_iter().map(Self::to_domain).collect())
    }

    #[instrument(
        name = "db.column.update",
        skip(self, column),
        fields(
            db.operation = "update",
            column.id = %column.id
        ),
        err
    )]
    async fn update(&self, column: Column) -> Result<Column, ApplicationError> {
        debug!("Updating column");
        let active_model = Self::to_active_model(column);

        let result = ColumnEntity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to update column");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(Self::to_domain(result))
    }

    #[instrument(
        name = "db.column.delete",
        skip(self, column_id),
        fields(
            db.operation = "delete",
            column.id = %column_id
        ),
        err
    )]
    async fn delete(&self, column_id: Uuid) -> Result<u64, ApplicationError> {
        debug!("Deleting column");
        let result = ColumnEntity::delete_by_id(column_id)
            .exec(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to delete column");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.rows_affected)
    }
}
