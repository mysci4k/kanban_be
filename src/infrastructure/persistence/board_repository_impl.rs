use crate::{
    domain::repositories::{Board, BoardRepository},
    shared::error::ApplicationError,
};
use async_trait::async_trait;
use entity::{BoardActiveModel, BoardEntity, BoardMemberColumn, BoardModel, BoardRelation};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait,
};
use tracing::{debug, error, instrument};
use uuid::Uuid;

pub struct SeaOrmBoardRepository {
    db: DatabaseConnection,
}

impl SeaOrmBoardRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn to_domain(model: BoardModel) -> Board {
        Board {
            id: model.id,
            name: model.name,
            description: model.description,
            owner_id: model.owner_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    fn to_active_model(board: Board) -> BoardActiveModel {
        BoardActiveModel {
            id: Set(board.id),
            name: Set(board.name),
            description: Set(board.description),
            owner_id: Set(board.owner_id),
            created_at: Set(board.created_at),
            updated_at: Set(board.updated_at),
        }
    }
}

#[async_trait]
impl BoardRepository for SeaOrmBoardRepository {
    #[instrument(
        name = "db.board.create",
        skip(self, board),
        fields(
            db.operation = "insert",
            board.id = %board.id
        ),
        err
    )]
    async fn create(&self, board: Board) -> Result<Board, ApplicationError> {
        debug!("Creating board");
        let active_model = Self::to_active_model(board);

        let result = BoardEntity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to create board");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(Self::to_domain(result))
    }

    #[instrument(
        name = "db.board.find_by_id",
        skip(self, board_id, user_id),
        fields(
            db.operation = "select",
            board.id = %board_id,
            user.id = %user_id
        ),
        err
    )]
    async fn find_by_id(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Board>, ApplicationError> {
        debug!("Finding board by ID");
        let result = BoardEntity::find_by_id(board_id)
            .join(JoinType::InnerJoin, BoardRelation::BoardMember.def())
            .filter(BoardMemberColumn::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to find board by ID");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.map(Self::to_domain))
    }

    #[instrument(
        name = "db.board.find_by_membership",
        skip(self, user_id),
        fields(
            db.operation = "select",
            user.id = %user_id
        ),
        err
    )]
    async fn find_by_membership(&self, user_id: Uuid) -> Result<Vec<Board>, ApplicationError> {
        debug!("Finding boards by user membership");
        let result = BoardEntity::find()
            .join(JoinType::InnerJoin, BoardRelation::BoardMember.def())
            .filter(BoardMemberColumn::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to find boards by user membership");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.into_iter().map(Self::to_domain).collect())
    }

    #[instrument(
        name = "db.board.update",
        skip(self, board),
        fields(
            db.operation = "update",
            board.id = %board.id
        ),
        err
    )]
    async fn update(&self, board: Board) -> Result<Board, ApplicationError> {
        debug!("Updating board");
        let active_model = Self::to_active_model(board);

        let result = BoardEntity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to update board");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(Self::to_domain(result))
    }

    #[instrument(
        name = "db.board.delete",
        skip(self, board_id),
        fields(
            db.operation = "delete",
            board.id = %board_id
        ),
        err
    )]
    async fn delete(&self, board_id: Uuid) -> Result<u64, ApplicationError> {
        debug!("Deleting board");
        let result = BoardEntity::delete_by_id(board_id)
            .exec(&self.db)
            .await
            .map_err(|err| {
                error!(error = %err, "Failed to delete board");
                ApplicationError::DatabaseError(err)
            })?;

        Ok(result.rows_affected)
    }
}
