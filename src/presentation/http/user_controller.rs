use crate::{
    application::{dto::UserDto, services::UserService},
    shared::{
        error::{ApplicationError, ApplicationErrorSchema},
        response::{ApiResponse, ApiResponseSchema},
    },
};
use actix_web::{get, web};
use std::sync::Arc;
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user").service(get_user_profile));
}

#[utoipa::path(
    get,
    description = "***PROTECTED ENDPOINT***\n\nRetrieves the complete profile information of the currently authenticated user.",
    path = "/user/profile",
    responses(
        (status = 200, description = "OK - User profile retrieved successfully", body = ApiResponseSchema<UserDto>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema),
        (status = 404, description = "Not found - User with the given ID not found", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to retrieve user profile", body = ApplicationErrorSchema)
    ),
    tag = "User",
    security(
        ("session_cookie" = [])
    )
)]
#[get("/profile")]
async fn get_user_profile(
    user_service: web::Data<Arc<UserService>>,
    user_id: web::ReqData<Uuid>,
) -> Result<ApiResponse<UserDto>, ApplicationError> {
    let user_id = user_id.into_inner();
    let user = user_service.get_user_by_id(user_id).await?;

    Ok(ApiResponse::Found {
        message: "User profile retrieved successfully".to_string(),
        data: user,
        page: None,
        total_pages: None,
    })
}
