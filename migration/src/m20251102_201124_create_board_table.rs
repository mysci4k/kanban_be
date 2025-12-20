use crate::m20251102_200527_create_user_table::User;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Board::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Board::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT uuidv7()"),
                    )
                    .col(ColumnDef::new(Board::Name).string_len(100).not_null())
                    .col(ColumnDef::new(Board::Description).text())
                    .col(ColumnDef::new(Board::OwnerId).uuid().not_null())
                    .col(
                        ColumnDef::new(Board::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        ColumnDef::new(Board::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_board_owner")
                            .from(Board::Table, Board::OwnerId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Board::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Board {
    Table,
    Id,
    Name,
    Description,
    OwnerId,
    CreatedAt,
    UpdatedAt,
}
