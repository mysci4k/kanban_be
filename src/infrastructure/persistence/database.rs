use crate::shared::utils::constants::DATABASE_URL;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, Statement};

pub async fn run() -> Result<DatabaseConnection, DbErr> {
    let database_url = DATABASE_URL.clone();
    let database_name = database_url.split('/').next_back().unwrap().to_string();

    let mut options = ConnectOptions::new(database_url);
    options.sqlx_logging(false);

    let database = Database::connect(options.clone()).await?;
    let database = match database.get_database_backend() {
        sea_orm::DatabaseBackend::MySql => {
            database
                .execute(Statement::from_string(
                    database.get_database_backend(),
                    format!("CREATE DATABASE IF NOT EXISTS '{}';", database_name),
                ))
                .await?;

            Database::connect(options).await?
        }
        sea_orm::DatabaseBackend::Postgres => {
            let res = database
                .execute(Statement::from_string(
                    database.get_database_backend(),
                    format!(
                        "SELECT datname FROM pg_catalog.pg_database WHERE datname = '{}';",
                        database_name
                    ),
                ))
                .await?;

            if res.rows_affected() == 0 {
                database
                    .execute(Statement::from_string(
                        database.get_database_backend(),
                        format!("CREATE DATABASE \"{}\"", database_name),
                    ))
                    .await?;
            }

            Database::connect(options).await?
        }
        sea_orm::DatabaseBackend::Sqlite => database,
    };

    Ok(database)
}
