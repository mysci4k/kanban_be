use crate::{m20251102_200527_create_user_table::User, m20251102_201124_create_board_table::Board};
use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::{extension::postgres::Type, *};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(BoardMemberRoleEnum)
                    .values(Role::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BoardMember::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BoardMember::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT uuidv7()"),
                    )
                    .col(ColumnDef::new(BoardMember::BoardId).uuid().not_null())
                    .col(ColumnDef::new(BoardMember::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(BoardMember::Role)
                            .enumeration(Alias::new("board_member_role_enum"), Role::iter())
                            .not_null()
                            .default(Role::Member.to_string()),
                    )
                    .col(
                        ColumnDef::new(BoardMember::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        ColumnDef::new(BoardMember::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .index(
                        Index::create()
                            .name("idx_board_members_board_user_unique")
                            .table(BoardMember::Table)
                            .col(BoardMember::BoardId)
                            .col(BoardMember::UserId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_board_member_board")
                            .from(BoardMember::Table, BoardMember::BoardId)
                            .to(Board::Table, Board::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_board_member_user")
                            .from(BoardMember::Table, BoardMember::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BoardMember::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum BoardMember {
    Table,
    Id,
    BoardId,
    UserId,
    Role,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub struct BoardMemberRoleEnum;

#[derive(EnumIter, Iden)]
pub enum Role {
    #[iden = "owner"]
    Owner,
    #[iden = "moderator"]
    Moderator,
    #[iden = "member"]
    Member,
}
