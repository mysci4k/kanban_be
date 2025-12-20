use crate::{
    application::services::{
        AuthService, BoardService, ColumnService, TaskService, UserService, WebSocketService,
    },
    domain::{
        events::SharedEventBus,
        repositories::{
            BoardMemberRepository, BoardRepository, ColumnRepository, TaskRepository,
            UserRepository,
        },
        services::{EmailService, TokenService},
    },
    infrastructure::{
        cache::RedisTokenService,
        email::SmtpEmailService,
        event_bus::InMemoryEventBus,
        persistence::{
            SeaOrmBoardMemberRepository, SeaOrmBoardRepository, SeaOrmColumnRepository,
            SeaOrmTaskRepository, SeaOrmUserRepository, database,
        },
    },
    shared::{config::AppState, utils::constants::REDIS_URL},
};
use redis::Client as RedisClient;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::info;

pub async fn initialize_infrastructure()
-> Result<(DatabaseConnection, RedisClient), Box<dyn std::error::Error>> {
    let database = database::run().await.map_err(|err| {
        eprintln!("Failed to connect to the database: {}", err);
        err
    })?;
    info!("Successfully connected to the PostgreSQL database");

    let redis_client = RedisClient::open(REDIS_URL.as_str()).map_err(|err| {
        eprintln!("Failed to create Redis client: {}", err);
        err
    })?;
    info!("Successfully connected to the Redis server");

    Ok((database, redis_client))
}

type Repositories = (
    Arc<dyn UserRepository>,
    Arc<dyn BoardRepository>,
    Arc<dyn BoardMemberRepository>,
    Arc<dyn ColumnRepository>,
    Arc<dyn TaskRepository>,
);

pub fn initialize_repositories(database: DatabaseConnection) -> Repositories {
    let user_repository =
        Arc::new(SeaOrmUserRepository::new(database.clone())) as Arc<dyn UserRepository>;
    let board_repository =
        Arc::new(SeaOrmBoardRepository::new(database.clone())) as Arc<dyn BoardRepository>;
    let board_member_repository = Arc::new(SeaOrmBoardMemberRepository::new(database.clone()))
        as Arc<dyn BoardMemberRepository>;
    let column_repository =
        Arc::new(SeaOrmColumnRepository::new(database.clone())) as Arc<dyn ColumnRepository>;
    let task_repository = Arc::new(SeaOrmTaskRepository::new(database)) as Arc<dyn TaskRepository>;

    info!("Successfully initialized repositories");

    (
        user_repository,
        board_repository,
        board_member_repository,
        column_repository,
        task_repository,
    )
}

pub fn initialize_event_bus() -> SharedEventBus {
    let event_bus = Arc::new(InMemoryEventBus::new()) as SharedEventBus;

    info!("Successfully initialized event bus");

    event_bus
}

pub fn initialize_services(
    user_repository: Arc<dyn UserRepository>,
    board_repository: Arc<dyn BoardRepository>,
    board_member_repository: Arc<dyn BoardMemberRepository>,
    column_repository: Arc<dyn ColumnRepository>,
    task_repository: Arc<dyn TaskRepository>,
    redis_client: RedisClient,
    event_bus: SharedEventBus,
) -> AppState {
    let token_service = Arc::new(RedisTokenService::new(redis_client)) as Arc<dyn TokenService>;
    let email_service =
        Arc::new(SmtpEmailService::new().expect("Failed to initialize email service"))
            as Arc<dyn EmailService>;

    let auth_service = Arc::new(AuthService::new(
        user_repository.clone(),
        token_service,
        email_service,
    ));
    let user_service = Arc::new(UserService::new(user_repository.clone()));
    let board_service = Arc::new(BoardService::new(
        user_repository,
        board_repository,
        board_member_repository.clone(),
        event_bus.clone(),
    ));
    let column_service = Arc::new(ColumnService::new(
        column_repository.clone(),
        board_member_repository.clone(),
        event_bus.clone(),
    ));
    let task_service = Arc::new(TaskService::new(
        task_repository,
        column_repository,
        board_member_repository.clone(),
        event_bus.clone(),
    ));
    let websocket_service = Arc::new(WebSocketService::new(event_bus, board_member_repository));

    info!("Successfully initialized services");

    AppState::new(
        auth_service,
        user_service,
        board_service,
        column_service,
        task_service,
        websocket_service,
    )
}
