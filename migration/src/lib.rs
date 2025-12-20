mod m20251102_200527_create_user_table;
mod m20251102_201124_create_board_table;
mod m20251102_201821_create_column_table;
mod m20251102_202640_create_task_table;
mod m20251108_111856_create_board_member_table;

pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251102_200527_create_user_table::Migration),
            Box::new(m20251102_201124_create_board_table::Migration),
            Box::new(m20251102_201821_create_column_table::Migration),
            Box::new(m20251102_202640_create_task_table::Migration),
            Box::new(m20251108_111856_create_board_member_table::Migration),
        ]
    }
}
