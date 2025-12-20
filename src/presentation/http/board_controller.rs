use crate::{
    application::{
        dto::{
            AddBoardMemberDto, BoardDto, BoardMemberDto, CreateBoardDto, DeleteBoardMemberDto,
            UpdateBoardDto, UpdateBoardMemberRoleDto,
        },
        services::BoardService,
    },
    shared::{
        error::{ApplicationError, ApplicationErrorSchema},
        response::{ApiResponse, ApiResponseSchema},
    },
};
use actix_web::{delete, get, post, put, web};
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/board")
            .service(create_board)
            .service(get_user_boards)
            .service(add_new_board_member)
            .service(update_board_member_role)
            .service(remove_board_member)
            .service(get_board)
            .service(update_board)
            .service(delete_board),
    );
}

#[utoipa::path(
    post,
    description = "***PROTECTED ENDPOINT***\n\nCreates a new Kanban board. The authenticated user automatically becomes the owner with full permissions.",
    path = "/board/",
    request_body = CreateBoardDto,
    responses(
        (status = 201, description = "Created - Board created successfully", body = ApiResponseSchema<BoardDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to create board", body = ApplicationErrorSchema)
    ),
    tag = "Board",
    security(
        ("session_cookie" = [])
    )
)]
#[post("/")]
async fn create_board(
    board_service: web::Data<Arc<BoardService>>,
    dto: web::Json<CreateBoardDto>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<BoardDto>, ApplicationError> {
    let user_id = user_id.into_inner();
    let board = board_service
        .create_board(dto.into_inner(), user_id)
        .await?;

    Ok(ApiResponse::Created {
        message: "Board created successfully".to_string(),
        data: board,
    })
}

#[utoipa::path(
    get,
    description = "***PROTECTED ENDPOINT***\n\nRetrieves detailed information about a specific board by its ID. Only board members can access this endpoint.",
    path = "/board/{boardId}",
    params(
        ("boardId" = Uuid, Path, description = "Unique identifier of the board")
    ),
    responses(
        (status = 200, description = "OK - Board data retrieved successfully", body = ApiResponseSchema<BoardDto>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 404, description = "Not found - Board with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to retrieve board", body = ApplicationErrorSchema)
    ),
    tag = "Board",
    security(
        ("session_cookie" = [])
    )
)]
#[get("/{boardId}")]
async fn get_board(
    board_service: web::Data<Arc<BoardService>>,
    board_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<BoardDto>, ApplicationError> {
    let board_id = board_id.into_inner();
    let user_id = user_id.into_inner();
    let board = board_service.get_board_by_id(board_id, user_id).await?;

    Ok(ApiResponse::Found {
        message: "Board data retrieved successfully".to_string(),
        data: board,
        page: None,
        total_pages: None,
    })
}

#[utoipa::path(
    get,
    description = "***PROTECTED ENDPOINT***\n\nRetrieves a list of all boards where the authenticated user is a member.",
    path = "/board/",
    responses(
        (status = 200, description = "OK - Boards retrieved successfully", body = ApiResponseSchema<Vec<BoardDto>>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to retrieve boards", body = ApplicationErrorSchema)
    ),
    tag = "Board",
    security(
        ("session_cookie" = [])
    )
)]
#[get("/")]
async fn get_user_boards(
    board_service: web::Data<Arc<BoardService>>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<Vec<BoardDto>>, ApplicationError> {
    let user_id = user_id.into_inner();
    let boards = board_service.get_boards_by_membership(user_id).await?;

    Ok(ApiResponse::Found {
        message: "Boards retrieved successfully".to_string(),
        data: boards,
        page: None,
        total_pages: None,
    })
}

#[utoipa::path(
    put,
    description = "***PROTECTED ENDPOINT***\n\nUpdates board information. Only the board owner and moderator can update board details.",
    path = "/board/{boardId}",
    params(
        ("boardId" = Uuid, Path, description = "Unique identifier of the board")
    ),
    request_body = UpdateBoardDto,
    responses(
        (status = 200, description = "OK - Board updated successfully", body = ApiResponseSchema<BoardDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - Only board owner and moderator can update board details", body = ApplicationErrorSchema),
        (status = 404, description = "Not found - Board with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to update board", body = ApplicationErrorSchema)
    ),
    tag = "Board",
    security(
        ("session_cookie" = [])
    )
)]
#[put("/{boardId}")]
async fn update_board(
    board_service: web::Data<Arc<BoardService>>,
    dto: web::Json<UpdateBoardDto>,
    board_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<BoardDto>, ApplicationError> {
    let board_id = board_id.into_inner();
    let user_id = user_id.into_inner();
    let board = board_service
        .update_board(dto.into_inner(), board_id, user_id)
        .await?;

    Ok(ApiResponse::Updated {
        message: "Board updated successfully".to_string(),
        data: board,
    })
}

#[utoipa::path(
    delete,
    description = "***PROTECTED ENDPOINT***\n\nPermanently deletes a board and all its associated data (columns, tasks, and members). Only the board owner can delete boards. This action cannot be undone.",
    path = "/board/{boardId}",
    params(
        ("boardId" = Uuid, Path, description = "Unique identifier of the board")
    ),
    responses(
        (status = 200, description = "OK - Board deleted successfully", body = ApiResponseSchema<u64>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - Only board owner can delete boards", body = ApplicationErrorSchema),
        (status = 404, description = "Not found - Board with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to delete board", body = ApplicationErrorSchema)
    ),
    tag = "Board",
    security(
        ("session_cookie" = [])
    )
)]
#[delete("/{boardId}")]
async fn delete_board(
    board_service: web::Data<Arc<BoardService>>,
    board_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<u64>, ApplicationError> {
    let board_id = board_id.into_inner();
    let user_id = user_id.into_inner();
    let rows_affected = board_service.delete_board(board_id, user_id).await?;

    Ok(ApiResponse::Deleted {
        message: "Board deleted successfully".to_string(),
        rows_affected,
    })
}

#[utoipa::path(
    post,
    description = "***PROTECTED ENDPOINT***\n\nAdds a new member to a board by user ID. Only the board owner and moderator can add new members. The new member will have the default member role and can be changed later.",
    path = "/board/member",
    request_body = AddBoardMemberDto,
    responses(
        (status = 201, description = "Created - Board member added successfully", body = ApiResponseSchema<BoardMemberDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - Only board owner and moderator can add members", body = ApplicationErrorSchema),
        (status = 404, description = "Not found - Board or user with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to add board member", body = ApplicationErrorSchema)
    ),
    tag = "Board",
    security(
        ("session_cookie" = [])
    )
)]
#[post("/member")]
async fn add_new_board_member(
    board_service: web::Data<Arc<BoardService>>,
    dto: web::Json<AddBoardMemberDto>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<BoardMemberDto>, ApplicationError> {
    let user_id = user_id.into_inner();
    let board_member = board_service
        .add_board_member(dto.into_inner(), user_id)
        .await?;

    Ok(ApiResponse::Created {
        message: "New board member added successfully".to_string(),
        data: board_member,
    })
}

#[utoipa::path(
    put,
    description = "***PROTECTED ENDPOINT***\n\nUpdates a board member's role. Only the board owner can modify member roles. The board owner cannot change their own role.",
    path = "/board/member",
    request_body = UpdateBoardMemberRoleDto,
    responses(
        (status = 200, description = "OK - Board member role updated successfully", body = ApiResponseSchema<BoardMemberDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - Only board owner can update member roles", body = ApplicationErrorSchema),
        (status = 404, description = "Not found - The specified user is not a member of this board", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to update board member role", body = ApplicationErrorSchema)
    ),
    tag = "Board",
    security(
        ("session_cookie" = [])
    )
)]
#[put("/member")]
async fn update_board_member_role(
    board_service: web::Data<Arc<BoardService>>,
    dto: web::Json<UpdateBoardMemberRoleDto>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<BoardMemberDto>, ApplicationError> {
    let user_id = user_id.into_inner();
    let board_member = board_service
        .update_board_member_role(dto.into_inner(), user_id)
        .await?;

    Ok(ApiResponse::Updated {
        message: "Board member role updated successfully".to_string(),
        data: board_member,
    })
}

#[utoipa::path(
    delete,
    description = "***PROTECTED ENDPOINT***\n\nRemoves a member from a board. Only the board owner can remove members. The board owner cannot remove themselves - to leave as owner, transfer ownership first or delete the board.",
    path = "/board/member",
    request_body = DeleteBoardMemberDto,
    responses(
        (status = 200, description = "OK - Board member removed successfully", body = ApiResponseSchema<u64>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - Only board owner can remove members", body = ApplicationErrorSchema),
        (status = 404, description = "Not found - The specified user is not a member of this board", body = ApplicationErrorSchema),
        (status = 409, description = "Conflict - Cannot remove yourself from the board", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to remove board member", body = ApplicationErrorSchema)
    ),
    tag = "Board",
    security(
        ("session_cookie" = [])
    )
)]
#[delete("/member")]
async fn remove_board_member(
    board_service: web::Data<Arc<BoardService>>,
    dto: web::Json<DeleteBoardMemberDto>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<u64>, ApplicationError> {
    let user_id = user_id.into_inner();
    let board_member = board_service
        .delete_board_member(dto.into_inner(), user_id)
        .await?;

    Ok(ApiResponse::Deleted {
        message: "Board member removed successfully".to_string(),
        rows_affected: board_member,
    })
}
