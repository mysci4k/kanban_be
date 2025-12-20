use crate::m20251102_201821_create_column_table::Column;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Task::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT uuidv7()"),
                    )
                    .col(ColumnDef::new(Task::Title).string_len(254).not_null())
                    .col(ColumnDef::new(Task::Description).text())
                    .col(ColumnDef::new(Task::Tags).array(ColumnType::String(StringLen::N(50))))
                    .col(ColumnDef::new(Task::Position).string_len(50).not_null())
                    .col(ColumnDef::new(Task::ColumnId).uuid().not_null())
                    .col(
                        ColumnDef::new(Task::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        ColumnDef::new(Task::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_column")
                            .from(Task::Table, Task::ColumnId)
                            .to(Column::Table, Column::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Task::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
    Title,
    Description,
    Tags,
    Position,
    ColumnId,
    CreatedAt,
    UpdatedAt,
}
