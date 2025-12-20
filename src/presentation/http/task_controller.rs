use crate::{
    application::{
        dto::{CreateTaskDto, TaskDto, UpdateTaskDto},
        services::TaskService,
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
        web::scope("/task")
            .service(create_task)
            .service(get_task)
            .service(get_column_tasks)
            .service(update_task)
            .service(move_task)
            .service(delete_task),
    );
}

#[utoipa::path(
    post,
    description = "***PROTECTED ENDPOINT***\n\nCreates a new task within a column. The task will be positioned at the end of the column. All board members can create tasks.",
    path = "/task/",
    request_body = CreateTaskDto,
    responses(
        (status = 201, description = "Created - Task created successfully", body = ApiResponseSchema<TaskDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have access to this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Column with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to create task", body = ApplicationErrorSchema)
    ),
    tag = "Task",
    security(
        ("session_cookie" = [])
    )
)]
#[post("/")]
async fn create_task(
    task_service: web::Data<Arc<TaskService>>,
    dto: web::Json<CreateTaskDto>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<TaskDto>, ApplicationError> {
    let user_id = user_id.into_inner();
    let task = task_service.create_task(dto.into_inner(), user_id).await?;

    Ok(ApiResponse::Created {
        message: "Task created successfully".to_string(),
        data: task,
    })
}

#[utoipa::path(
    get,
    description = "***PROTECTED ENDPOINT***\n\nRetrieves detailed information about a specific task by its ID. User must be a member of the board to access this endpoint.",
    path = "/task/{taskId}",
    params(
        ("taskId" = Uuid, Path, description = "Unique identifier of the task")
    ),
    responses(
        (status = 200, description = "OK - Task data retrieved successfully", body = ApiResponseSchema<TaskDto>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have access to this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Task with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to retrieve task", body = ApplicationErrorSchema)
    ),
    tag = "Task",
    security(
        ("session_cookie" = [])
    )
)]
#[get("/{taskId}")]
async fn get_task(
    task_service: web::Data<Arc<TaskService>>,
    task_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<TaskDto>, ApplicationError> {
    let task_id = task_id.into_inner();
    let user_id = user_id.into_inner();
    let task = task_service.get_task_by_id(task_id, user_id).await?;

    Ok(ApiResponse::Found {
        message: "Task data retrieved successfully".to_string(),
        data: task,
        page: None,
        total_pages: None,
    })
}

#[utoipa::path(
    get,
    description = "***PROTECTED ENDPOINT***\n\nRetrieves all tasks for a specific column, ordered by their position. User must be a member of the board to access this endpoint.",
    path = "/task/column/{columnId}",
    params(
        ("columnId" = Uuid, Path, description = "Unique identifier of the column")
    ),
    responses(
        (status = 200, description = "OK - Tasks retrieved successfully", body = ApiResponseSchema<Vec<TaskDto>>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have access to this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Column with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to retrieve tasks", body = ApplicationErrorSchema)
    ),
    tag = "Task",
    security(
        ("session_cookie" = [])
    )
)]
#[get("/column/{columnId}")]
async fn get_column_tasks(
    task_service: web::Data<Arc<TaskService>>,
    column_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<Vec<TaskDto>>, ApplicationError> {
    let column_id = column_id.into_inner();
    let user_id = user_id.into_inner();
    let tasks = task_service.get_column_tasks(column_id, user_id).await?;

    Ok(ApiResponse::Found {
        message: "Tasks retrieved successfully".to_string(),
        data: tasks,
        page: None,
        total_pages: None,
    })
}

#[utoipa::path(
    put,
    description = "***PROTECTED ENDPOINT***\n\nUpdates task information. All board members can update tasks.",
    path = "/task/{taskId}",
    params(
        ("taskId" = Uuid, Path, description = "Unique identifier of the task")
    ),
    request_body = UpdateTaskDto,
    responses(
        (status = 200, description = "OK - Task updated successfully", body = ApiResponseSchema<TaskDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have access to this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Task with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to update task", body = ApplicationErrorSchema)
    ),
    tag = "Task",
    security(
        ("session_cookie" = [])
    )
)]
#[put("/{taskId}")]
async fn update_task(
    task_service: web::Data<Arc<TaskService>>,
    dto: web::Json<UpdateTaskDto>,
    task_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<TaskDto>, ApplicationError> {
    let task_id = task_id.into_inner();
    let user_id = user_id.into_inner();
    let task = task_service
        .update_task(dto.into_inner(), task_id, user_id)
        .await?;

    Ok(ApiResponse::Updated {
        message: "Task updated successfully".to_string(),
        data: task,
    })
}

#[utoipa::path(
    put,
    description = "***PROTECTED ENDPOINT***\n\nMoves a task to a new position within the same column or to a different column. Position is 0-indexed. All board members can move tasks.",
    path = "/task/{taskId}/move/{columnId}/{position}",
    params(
        ("taskId" = Uuid, Path, description = "Unique identifier of the task"),
        ("columnId" = Uuid, Path, description = "Unique identifier of the column"),
        ("position" = usize, Path, description = "New position index for the column (0-based)")
    ),
    responses(
        (status = 200, description = "OK - Task moved successfully", body = ApiResponseSchema<TaskDto>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have access to this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Task or column with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to move task", body = ApplicationErrorSchema)
    ),
    tag = "Task",
    security(
        ("session_cookie" = [])
    )
)]
#[put("/{taskId}/move/{columnId}/{position}")]
async fn move_task(
    task_service: web::Data<Arc<TaskService>>,
    path: web::Path<(Uuid, Uuid, usize)>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<TaskDto>, ApplicationError> {
    let (task_id, column_id, position) = path.into_inner();
    let user_id = user_id.into_inner();
    let task = task_service
        .move_task(position, task_id, column_id, user_id)
        .await?;

    Ok(ApiResponse::Updated {
        message: "Task moved successfully".to_string(),
        data: task,
    })
}

#[utoipa::path(
    delete,
    description = "***PROTECTED ENDPOINT***\n\nPermanently deletes a task. This action cannot be undone. All board members can delete tasks.",
    path = "/task/{taskId}",
    params(
        ("taskId" = Uuid, Path, description = "Unique identifier of the task")
    ),
    responses(
        (status = 200, description = "OK - Task deleted successfully", body = ApiResponseSchema<u64>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - User doesn't have access to this board", body = ApplicationErrorSchema),
        (status = 404, description = "Not Found - Task with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to delete task", body = ApplicationErrorSchema)
    ),
    tag = "Task",
    security(
        ("session_cookie" = [])
    )
)]
#[delete("/{taskId}")]
async fn delete_task(
    task_service: web::Data<Arc<TaskService>>,
    task_id: web::Path<Uuid>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<u64>, ApplicationError> {
    let task_id = task_id.into_inner();
    let user_id = user_id.into_inner();
    let rows_affected = task_service.delete_task(task_id, user_id).await?;

    Ok(ApiResponse::Deleted {
        message: "Task deleted successfully".to_string(),
        rows_affected,
    })
}
