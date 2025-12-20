use crate::{
    application::services::WebSocketService,
    shared::error::{ApplicationError, ApplicationErrorSchema},
};
use actix_web::{
    HttpRequest, Responder, get,
    web::{self, Payload},
};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/ws").service(websocket_handler));
}

#[utoipa::path(
    get,
    description = "***PROTECTED ENDPOINT***\n\nEstablishes a WebSocket connection for real-time board updates. The connection enables bidirectional communication for live collaboration features such as task updates, column changes, and member activities. User must be a member of the board to establish the connection.",
    path = "/ws/board/{boardId}",
    params(
        ("boardId" = Uuid, Path, description = "Unique identifier of the board")
    ),
    responses(
        (status = 101, description = "Switching Protocols - WebSocket connection established successfully"),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User is not a member of this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Board with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to establish WebSocket connection", body = ApplicationErrorSchema)
    ),
    tag = "WebSocket",
    security(
        ("session_cookie" = [])
    )
)]
#[get("/board/{boardId}")]
async fn websocket_handler(
    websocket_service: web::Data<Arc<WebSocketService>>,
    req: HttpRequest,
    stream: Payload,
    board_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<impl Responder, ApplicationError> {
    let board_id = board_id.into_inner();
    let user_id = user_id.into_inner();

    websocket_service
        .verify_board_access(board_id, user_id)
        .await?;

    let (response, session, msg_stream) = actix_ws::handle(&req, stream).map_err(|err| {
        error!("WebSocket handshake error: {}", err);
        ApplicationError::InternalError {
            message: "Failed to establish WebSocket connection".to_string(),
        }
    })?;

    actix_web::rt::spawn(async move {
        websocket_service
            .handle_connection(board_id, user_id, session, msg_stream)
            .await;
    });

    Ok(response)
}
