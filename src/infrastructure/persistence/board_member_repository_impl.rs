use crate::{
    domain::repositories::{BoardMember, BoardMemberRepository},
    shared::error::ApplicationError,
};
use async_trait::async_trait;
use entity::{
    BoardColumn, BoardEntity, BoardMemberActiveModel, BoardMemberColumn, BoardMemberEntity,
    BoardMemberModel, BoardMemberRoleEnum,
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    FromQueryResult, QueryFilter,
};
use sea_query::{Alias, Expr, ExprTrait, Query};
use uuid::Uuid;

pub struct SeaOrmBoardMemberRepository {
    db: DatabaseConnection,
}

impl SeaOrmBoardMemberRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn to_domain(model: BoardMemberModel) -> BoardMember {
        BoardMember {
            id: model.id,
            board_id: model.board_id,
            user_id: model.user_id,
            role: model.role,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    fn to_active_model(board_member: BoardMember) -> BoardMemberActiveModel {
        BoardMemberActiveModel {
            id: Set(board_member.id),
            board_id: Set(board_member.board_id),
            user_id: Set(board_member.user_id),
            role: Set(board_member.role),
            created_at: Set(board_member.created_at),
            updated_at: Set(board_member.updated_at),
        }
    }
}

#[async_trait]
impl BoardMemberRepository for SeaOrmBoardMemberRepository {
    async fn create(&self, board_member: BoardMember) -> Result<BoardMember, ApplicationError> {
        let active_model = Self::to_active_model(board_member);

        let result = BoardMemberEntity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(Self::to_domain(result))
    }

    async fn find_by_board_and_user_id(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BoardMember>, ApplicationError> {
        let result = BoardMemberEntity::find()
            .filter(BoardMemberColumn::BoardId.eq(board_id))
            .filter(BoardMemberColumn::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(result.map(Self::to_domain))
    }

    async fn get_role(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BoardMemberRoleEnum>, ApplicationError> {
        let result = BoardMemberEntity::find()
            .filter(BoardMemberColumn::BoardId.eq(board_id))
            .filter(BoardMemberColumn::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(result.map(|m| m.role))
    }

    async fn check_permissions(
        &self,
        board_id: Uuid,
        user_id: Uuid,
        member_roles: Vec<BoardMemberRoleEnum>,
    ) -> Result<bool, ApplicationError> {
        #[derive(FromQueryResult)]
        struct PermissionCheck {
            board_exists: bool,
            has_permission: bool,
        }

        let board_exists_subquery = Query::select()
            .expr(Expr::value(1))
            .from(BoardEntity)
            .and_where(Expr::col((BoardEntity, BoardColumn::Id)).eq(board_id))
            .to_owned();

        let permission_subquery = Query::select()
            .expr(Expr::value(1))
            .from(BoardMemberEntity)
            .and_where(Expr::col((BoardMemberEntity, BoardMemberColumn::BoardId)).eq(board_id))
            .and_where(Expr::col((BoardMemberEntity, BoardMemberColumn::UserId)).eq(user_id))
            .and_where(
                Expr::col((BoardMemberEntity, BoardMemberColumn::Role))
                    .cast_as(Alias::new("text"))
                    .is_in(member_roles),
            )
            .to_owned();

        let query = Query::select()
            .expr_as(
                Expr::exists(board_exists_subquery),
                Alias::new("board_exists"),
            )
            .expr_as(
                Expr::exists(permission_subquery),
                Alias::new("has_permission"),
            )
            .to_owned();

        let builder = self.db.get_database_backend();
        let statement = builder.build(&query);

        let result = PermissionCheck::find_by_statement(statement)
            .one(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?
            .ok_or_else(|| {
                ApplicationError::DatabaseError(DbErr::RecordNotFound(
                    "Query returned no results".to_string(),
                ))
            })?;

        if !result.board_exists {
            return Err(ApplicationError::NotFound {
                message: "Board with the given ID not found".to_string(),
            });
        }

        Ok(result.has_permission)
    }

    async fn update(&self, board_member: BoardMember) -> Result<BoardMember, ApplicationError> {
        let active_model = Self::to_active_model(board_member);

        let result = BoardMemberEntity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(Self::to_domain(result))
    }

    async fn delete(&self, board_id: Uuid, user_id: Uuid) -> Result<u64, ApplicationError> {
        let result = BoardMemberEntity::delete_many()
            .filter(BoardMemberColumn::BoardId.eq(board_id))
            .filter(BoardMemberColumn::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(ApplicationError::DatabaseError)?;

        Ok(result.rows_affected)
    }
}
