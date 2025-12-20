use crate::{
    application::{
        dto::{ColumnDto, CreateColumnDto, UpdateColumnDto},
        services::ColumnService,
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
        web::scope("/column")
            .service(create_column)
            .service(get_column)
            .service(get_board_columns)
            .service(update_column)
            .service(move_column)
            .service(delete_column),
    );
}

#[utoipa::path(
    post,
    description = "***PROTECTED ENDPOINT***\n\nCreates a new column within a board. The column will be positioned at the end of the board. Only the board owner and moderator can create columns.",
    path = "/column/",
    request_body = CreateColumnDto,
    responses(
        (status = 201, description = "Created - Column created successfully", body = ApiResponseSchema<ColumnDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have permission to create columns in this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Board with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to create column", body = ApplicationErrorSchema)
    ),
    tag = "Column",
    security(
        ("session_cookie" = [])
    )
 )]
#[post("/")]
async fn create_column(
    column_service: web::Data<Arc<ColumnService>>,
    dto: web::Json<CreateColumnDto>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<ColumnDto>, ApplicationError> {
    let user_id = user_id.into_inner();
    let column = column_service
        .create_column(dto.into_inner(), user_id)
        .await?;

    Ok(ApiResponse::Created {
        message: "Column created successfully".to_string(),
        data: column,
    })
}

#[utoipa::path(
    get,
    description = "***PROTECTED ENDPOINT***\n\nRetrieves detailed information about a specific column by its ID. User must be a member of the board to access this endpoint.",
    path = "/column/{columnId}",
    params(
        ("columnId" = Uuid, Path, description = "Unique identifier of the column")
    ),
    responses(
        (status = 200, description = "OK - Column data retrieved successfully", body = ApiResponseSchema<ColumnDto>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have access to the board containing this column", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Column with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to retrieve column", body = ApplicationErrorSchema)
    ),
    tag = "Column",
    security(
        ("session_cookie" = [])
    )
 )]
#[get("/{columnId}")]
async fn get_column(
    column_service: web::Data<Arc<ColumnService>>,
    column_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<ColumnDto>, ApplicationError> {
    let user_id = user_id.into_inner();
    let column = column_service
        .get_column_by_id(column_id.into_inner(), user_id)
        .await?;

    Ok(ApiResponse::Found {
        message: "Column data retrieved successfully".to_string(),
        data: column,
        page: None,
        total_pages: None,
    })
}

#[utoipa::path(
    get,
    description = "***PROTECTED ENDPOINT***\n\nRetrieves all columns for a specific board, ordered by their position. User must be a member of the board to access this endpoint.",
    path = "/column/board/{boardId}",
    params(
        ("boardId" = Uuid, Path, description = "Unique identifier of the board")
    ),
    responses(
        (status = 200, description = "OK - Columns retrieved successfully", body = ApiResponseSchema<Vec<ColumnDto>>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have access to this board", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to retrieve columns", body = ApplicationErrorSchema)
    ),
    tag = "Column",
    security(
        ("session_cookie" = [])
    )
 )]
#[get("/board/{boardId}")]
async fn get_board_columns(
    column_service: web::Data<Arc<ColumnService>>,
    board_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<Vec<ColumnDto>>, ApplicationError> {
    let user_id = user_id.into_inner();
    let columns = column_service
        .get_board_columns(board_id.into_inner(), user_id)
        .await?;

    Ok(ApiResponse::Found {
        message: "Columns retrieved successfully".to_string(),
        data: columns,
        page: None,
        total_pages: None,
    })
}

#[utoipa::path(
    put,
    description = "***PROTECTED ENDPOINT***\n\nUpdates column information. Only the board owner and moderator can update column details.",
    path = "/column/{columnId}",
    params(
        ("columnId" = Uuid, Path, description = "Unique identifier of the column")
    ),
    request_body = UpdateColumnDto,
    responses(
        (status = 200, description = "OK - Column updated successfully", body = ApiResponseSchema<ColumnDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have permission to update columns in this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Column with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to update column", body = ApplicationErrorSchema)
    ),
    tag = "Column",
    security(
        ("session_cookie" = [])
    )
 )]
#[put("/{columnId}")]
async fn update_column(
    column_service: web::Data<Arc<ColumnService>>,
    dto: web::Json<UpdateColumnDto>,
    column_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<ColumnDto>, ApplicationError> {
    let column_id = column_id.into_inner();
    let user_id = user_id.into_inner();
    let column = column_service
        .update_column(dto.into_inner(), column_id, user_id)
        .await?;

    Ok(ApiResponse::Updated {
        message: "Column updated successfully".to_string(),
        data: column,
    })
}

#[utoipa::path(
    put,
    description = "***PROTECTED ENDPOINT***\n\nMoves a column to a new position within the board. Other columns will be automatically reordered. Position is 0-indexed. Only the board owner and moderator can create columns.",
    path = "/column/{columnId}/move/{position}",
    params(
        ("columnId" = Uuid, Path, description = "Unique identifier of the column"),
        ("position" = usize, Path, description = "New position index for the column (0-based)")
    ),
    responses(
        (status = 200, description = "OK - Column moved successfully", body = ApiResponseSchema<ColumnDto>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have permission to reorder columns in this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Column with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to move column", body = ApplicationErrorSchema)
    ),
    tag = "Column",
    security(
        ("session_cookie" = [])
    )
 )]
#[put("/{columnId}/move/{position}")]
async fn move_column(
    column_service: web::Data<Arc<ColumnService>>,
    path: web::Path<(Uuid, usize)>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<ColumnDto>, ApplicationError> {
    let (column_id, position) = path.into_inner();
    let user_id = user_id.into_inner();
    let column = column_service
        .move_column(position, column_id, user_id)
        .await?;

    Ok(ApiResponse::Updated {
        message: "Column moved successfully".to_string(),
        data: column,
    })
}

#[utoipa::path(
    delete,
    description = "***PROTECTED ENDPOINT***\n\nPermanently deletes a column and all its associated tasks. Remaining columns will be automatically reordered. This action cannot be undone. Only the board owner and moderator can create columns.",
    path = "/column/{columnId}",
    params(
        ("columnId" = Uuid, Path, description = "Unique identifier of the column")
    ),
    responses(
        (status = 200, description = "OK - Column deleted successfully", body = ApiResponseSchema<u64>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have permission to delete columns in this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Column with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to delete column", body = ApplicationErrorSchema)
    ),
    tag = "Column",
    security(
        ("session_cookie" = [])
    )
 )]
#[delete("/{columnId}")]
async fn delete_column(
    column_service: web::Data<Arc<ColumnService>>,
    column_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<()>, ApplicationError> {
    let column_id = column_id.into_inner();
    let user_id = user_id.into_inner();
    let rows_affected = column_service.delete_column(column_id, user_id).await?;

    Ok(ApiResponse::Deleted {
        message: "Column deleted successfully".to_string(),
        rows_affected,
    })
}
