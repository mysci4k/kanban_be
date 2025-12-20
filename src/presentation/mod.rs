pub mod http;
pub mod middleware;

pub use http::configure_auth_roures;
pub use http::configure_board_routes;
pub use http::configure_column_routes;
pub use http::configure_task_routes;
pub use http::configure_user_routes;
pub use http::configure_websocket_routes;
