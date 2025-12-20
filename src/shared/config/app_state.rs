use crate::application::services::{
    AuthService, BoardService, ColumnService, TaskService, UserService, WebSocketService,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub board_service: Arc<BoardService>,
    pub column_service: Arc<ColumnService>,
    pub task_service: Arc<TaskService>,
    pub websocket_service: Arc<WebSocketService>,
}

impl AppState {
    pub fn new(
        auth_service: Arc<AuthService>,
        user_service: Arc<UserService>,
        board_service: Arc<BoardService>,
        column_service: Arc<ColumnService>,
        task_service: Arc<TaskService>,
        websocket_service: Arc<WebSocketService>,
    ) -> Self {
        Self {
            auth_service,
            user_service,
            board_service,
            column_service,
            task_service,
            websocket_service,
        }
    }
}
