use crate::application::dto::{
    ActivationQueryDto, AddBoardMemberDto, BoardDto, BoardMemberDto, ColumnDto, CreateBoardDto,
    CreateColumnDto, CreateTaskDto, CreateUserDto, DeleteBoardMemberDto, ForgotPasswordQueryDto,
    LoginDto, ResendActivationQueryDto, ResetPasswordDto, TaskDto, UpdateBoardDto,
    UpdateBoardMemberRoleDto, UpdateColumnDto, UpdateTaskDto, UserDto,
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Auth endpoints
        crate::presentation::http::auth_controller::register,
        crate::presentation::http::auth_controller::login,
        crate::presentation::http::auth_controller::logout,
        crate::presentation::http::auth_controller::activate,
        crate::presentation::http::auth_controller::resend_activation,
        crate::presentation::http::auth_controller::forgot_password,
        crate::presentation::http::auth_controller::reset_password,

        // User endpoints
        crate::presentation::http::user_controller::get_user_profile,

        // Board endpoints
        crate::presentation::http::board_controller::create_board,
        crate::presentation::http::board_controller::get_board,
        crate::presentation::http::board_controller::get_user_boards,
        crate::presentation::http::board_controller::update_board,
        crate::presentation::http::board_controller::delete_board,
        crate::presentation::http::board_controller::remove_board_member,
        crate::presentation::http::board_controller::update_board_member_role,
        crate::presentation::http::board_controller::add_new_board_member,

        // Column endpoints
        crate::presentation::http::column_controller::create_column,
        crate::presentation::http::column_controller::get_column,
        crate::presentation::http::column_controller::get_board_columns,
        crate::presentation::http::column_controller::update_column,
        crate::presentation::http::column_controller::move_column,
        crate::presentation::http::column_controller::delete_column,

        // Task endpoints
        crate::presentation::http::task_controller::create_task,
        crate::presentation::http::task_controller::get_task,
        crate::presentation::http::task_controller::get_column_tasks,
        crate::presentation::http::task_controller::update_task,
        crate::presentation::http::task_controller::move_task,
        crate::presentation::http::task_controller::delete_task,

        // Websocket endpoints
        crate::presentation::http::websocket_controller::websocket_handler
    ),
    components(
        schemas(
            // Auth DTOs
            LoginDto,
            ActivationQueryDto,
            ResendActivationQueryDto,
            ForgotPasswordQueryDto,
            ResetPasswordDto,

            // User DTOs
            UserDto,
            CreateUserDto,

            // Board DTOs
            BoardDto,
            CreateBoardDto,
            UpdateBoardDto,

            // Board member DTOs
            BoardMemberDto,
            AddBoardMemberDto,
            UpdateBoardMemberRoleDto,
            DeleteBoardMemberDto,

            // Column DTOs
            ColumnDto,
            CreateColumnDto,
            UpdateColumnDto,

            // Task DTOs
            TaskDto,
            CreateTaskDto,
            UpdateTaskDto
        )
    ),
    tags(
        (name = "Authentication", description = "Authentication management endpoints."),
        (name = "User", description = "User management endpoints."),
        (name = "Board", description = "Board management endpoints."),
        (name = "Column", description = "Column management endpoints."),
        (name = "Task", description = "Task management endpoints."),
        (name = "WebSocket", description = "WebSocket management endpoints.")
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Kanban Board API",
        version = "0.1.0",
        description = "REST API backend for a Kanban application.",
        license(
            name = "MIT",
            url = "https://opensource.org/license/mit/"
        ),
        contact(
            name = "API Support",
            url = "https://github.com/mysci4k/kanban_be"
        )
    ),
    servers(
        (url = "http://localhost:8080/api", description = "Local development server")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "session_cookie",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("user-session"))),
            );
        }
    }
}
