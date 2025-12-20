pub mod auth_controller;
pub mod board_controller;
pub mod column_controller;
pub mod openapi;
pub mod server;
pub mod task_controller;
pub mod user_controller;
pub mod websocket_controller;

pub use auth_controller::configure as configure_auth_roures;
pub use board_controller::configure as configure_board_routes;
pub use column_controller::configure as configure_column_routes;
pub use openapi::ApiDoc;
pub use server::configure_server;
pub use task_controller::configure as configure_task_routes;
pub use user_controller::configure as configure_user_routes;
pub use websocket_controller::configure as configure_websocket_routes;
