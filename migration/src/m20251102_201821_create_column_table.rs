use crate::m20251102_201124_create_board_table::Board;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Column::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Column::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT uuidv7()"),
                    )
                    .col(ColumnDef::new(Column::Name).string_len(100).not_null())
                    .col(ColumnDef::new(Column::Position).string_len(50).not_null())
                    .col(ColumnDef::new(Column::BoardId).uuid().not_null())
                    .col(
                        ColumnDef::new(Column::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        ColumnDef::new(Column::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_column_board")
                            .from(Column::Table, Column::BoardId)
                            .to(Board::Table, Board::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Column::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Column {
    Table,
    Id,
    Name,
    Position,
    BoardId,
    CreatedAt,
    UpdatedAt,
}
