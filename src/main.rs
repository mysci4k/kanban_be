mod application;
mod domain;
mod infrastructure;
mod presentation;
mod shared;

use crate::{
    presentation::http::configure_server,
    shared::{
        config::{
            TracingConfig, init_tracing, initialize_event_bus, initialize_infrastructure,
            initialize_repositories, initialize_services,
        },
        utils::constants::{SERVER_ADDRESS, SERVER_PORT},
    },
};
use dotenvy::dotenv;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let tracing_config = TracingConfig::from_env();
    let _tracing_guard = init_tracing(tracing_config);

    let (database, redis_client) = initialize_infrastructure()
        .await
        .expect("Failed to initialize infrastructure");

    let (
        user_repository,
        board_repository,
        board_member_repository,
        column_repository,
        task_repository,
    ) = initialize_repositories(database);

    let event_bus = initialize_event_bus();

    let app_state = initialize_services(
        user_repository,
        board_repository,
        board_member_repository,
        column_repository,
        task_repository,
        redis_client,
        event_bus,
    );

    let server = configure_server(app_state, &SERVER_ADDRESS, *SERVER_PORT).await?;

    server.await
}
