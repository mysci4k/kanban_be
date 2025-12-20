use crate::{
    application::{
        dto::{
            ActivationQueryDto, CreateUserDto, ForgotPasswordQueryDto, LoginDto,
            ResendActivationQueryDto, ResetPasswordDto, UserDto,
        },
        services::AuthService,
    },
    shared::{
        error::{ApplicationError, ApplicationErrorSchema},
        response::{ApiResponse, ApiResponseSchema},
    },
};
use actix_identity::Identity;
use actix_web::{HttpMessage, HttpRequest, post, web};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(logout)
            .service(activate)
            .service(resend_activation)
            .service(forgot_password)
            .service(reset_password),
    );
}

#[utoipa::path(
    post,
    description = "***PUBLIC ENDPOINT***\n\nRegisters a new user account. After successful registration, an activation email will be sent to the provided email address. The user must activate their account before being able to log in.",
    path = "/auth/register",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "Created - User registered successfully. An activation email has been sent", body = ApiResponseSchema<UserDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 409, description = "Conflict - User with this email address already exists", body = ApplicationErrorSchema),
        (status = 500, description = "Internal Server Error - Failed to create user or send activation email", body = ApplicationErrorSchema)
    ),
    tag = "Authentication"
)]
#[post("/register")]
async fn register(
    auth_service: web::Data<Arc<AuthService>>,
    dto: web::Json<CreateUserDto>,
) -> Result<ApiResponse<UserDto>, ApplicationError> {
    let user = auth_service.register(dto.into_inner()).await?;

    Ok(ApiResponse::Created {
        message: "User registered successfully. An activation email has been sent".to_string(),
        data: user,
    })
}

#[utoipa::path(
    post,
    description = "***PUBLIC ENDPOINT***\n\nAuthenticates a user with email and password credentials. Upon successful authentication, a session cookie is created and returned in the response headers. The user must have an activated account to log in.",
    path = "/auth/login",
    request_body = LoginDto,
    responses(
        (status = 200, description = "OK - User logged in successfully. Session cookie has been set", body = ApiResponseSchema<UserDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - Invalid credentials", body = ApplicationErrorSchema),
        (status = 403, description = "Forbidden - Account is not activated", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to create user session", body = ApplicationErrorSchema)
    ),
    tag = "Authentication"
)]
#[post("/login")]
async fn login(
    auth_service: web::Data<Arc<AuthService>>,
    dto: web::Json<LoginDto>,
    req: HttpRequest,
) -> Result<ApiResponse<UserDto>, ApplicationError> {
    let user = auth_service.login(dto.into_inner()).await?;

    Identity::login(&req.extensions(), user.id.to_string()).map_err(|_| {
        ApplicationError::InternalError {
            message: "Failed to create user session".to_string(),
        }
    })?;

    Ok(ApiResponse::Ok {
        message: "User logged in successfully. Session cookie has been set".to_string(),
        data: Some(user),
    })
}

#[utoipa::path(
    post,
    description = "***PROTECTED ENDPOINT***\n\nTerminates the current user session by invalidating the session cookie. The user will need to log in again to access protected endpoints.",
    path = "/auth/logout",
    responses(
        (status = 200, description = "OK - User logged out successfully. Session has been terminated", body = ApiResponseSchema<String>),
        (status = 401, description = "Unauthorized - No active session or session has expired", body = ApplicationErrorSchema)
    ),
    tag = "Authentication",
    security(
        ("session_cookie" = [])
    )
)]
#[post("/logout")]
async fn logout(identity: Identity) -> Result<ApiResponse<String>, ApplicationError> {
    identity.logout();

    Ok(ApiResponse::Ok {
        message: "User logged out successfully. Session has been terminated".to_string(),
        data: None,
    })
}

#[utoipa::path(
    post,
    description = "***PUBLIC ENDPOINT***\n\nActivates a user account using the activation token sent via email during registration. Once activated, the user can log in to the application.",
    path = "/auth/activate",
    params(
        ("userId" = String, Query, description = "Unique identifier of the user"),
        ("activationToken" = String, Query, description = "Unique activation token")
    ),
    responses(
        (status = 200, description = "OK - Account activated successfully", body = ApiResponseSchema<UserDto>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to activate account", body = ApplicationErrorSchema)
    ),
    tag = "Authentication"
)]
#[post("/activate")]
async fn activate(
    auth_service: web::Data<Arc<AuthService>>,
    query: web::Query<ActivationQueryDto>,
) -> Result<ApiResponse<UserDto>, ApplicationError> {
    let user = auth_service
        .activate_user(query.user_id.clone(), query.activation_token.clone())
        .await?;

    Ok(ApiResponse::Ok {
        message: "Account activated successfully".to_string(),
        data: Some(user),
    })
}

#[utoipa::path(
    post,
    description = "***PUBLIC ENDPOINT***\n\nResends the account activation email to the specified email address when the original activation email was not received or has expired.",
    path = "/auth/resend-activation",
    params(
        ("email" = String, Query, description = "Email address of the user")
    ),
    responses(
        (status = 200, description = "OK - Activation email resent successfully", body = ApiResponseSchema<String>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 404, description = "Not found - User with the given email address not found", body = ApplicationErrorSchema),
        (status = 409, description = "Conflict - Account is already activated", body = ApplicationErrorSchema),
        (status = 429, description = "Too Many Requests - Activation email resend limit exceeded", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to send activation email", body = ApplicationErrorSchema)
    ),
    tag = "Authentication"
)]
#[post("/resend-activation")]
async fn resend_activation(
    auth_service: web::Data<Arc<AuthService>>,
    query: web::Query<ResendActivationQueryDto>,
) -> Result<ApiResponse<String>, ApplicationError> {
    auth_service
        .resend_activation_email(query.email.clone())
        .await?;

    Ok(ApiResponse::Ok {
        message: "Activation email resent successfully".to_string(),
        data: None,
    })
}

#[utoipa::path(
    post,
    description = "***PUBLIC ENDPOINT***\n\nInitiates the password reset process by sending a password reset email to the specified email address. The email contains a unique reset token that can be used to set a new password.",
    path = "/auth/forgot-password",
    params(
        ("email" = String, Query, description = "Email address of the user")
    ),
    responses(
        (status = 200, description = "OK - Password reset email sent successfully. Check your inbox for further instructions.", body = ApiResponseSchema<String>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 401, description = "Unauthorized - Account is not activated", body = ApplicationErrorSchema),
        (status = 404, description = "Not found - User with the given email address not found", body = ApplicationErrorSchema),
        (status = 429, description = "Too Many Requests - Password reset email request limit exceeded", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to send password reset email", body = ApplicationErrorSchema)
    ),
    tag = "Authentication"
)]
#[post("/forgot-password")]
async fn forgot_password(
    auth_service: web::Data<Arc<AuthService>>,
    query: web::Query<ForgotPasswordQueryDto>,
) -> Result<ApiResponse<String>, ApplicationError> {
    auth_service.forgot_password(query.email.clone()).await?;

    Ok(ApiResponse::Ok {
        message: "Password reset email sent successfully".to_string(),
        data: None,
    })
}

#[utoipa::path(
    post,
    description = "***PUBLIC ENDPOINT***\n\nCompletes the password reset process by setting a new password using the reset token received via email.",
    path = "/auth/reset-password",
    request_body = ResetPasswordDto,
    responses(
        (status = 200, description = "OK - Password reset successfully. You can now log in with your new password", body = ApiResponseSchema<String>),
        (status = 400, description = "Bad Request - Invalid input data", body = ApplicationErrorSchema),
        (status = 500, description = "Internal server error - Failed to reset password", body = ApplicationErrorSchema)
    ),
    tag = "Authentication"
)]
#[post("/reset-password")]
async fn reset_password(
    auth_service: web::Data<Arc<AuthService>>,
    dto: web::Json<ResetPasswordDto>,
) -> Result<ApiResponse<String>, ApplicationError> {
    auth_service.reset_password(dto.into_inner()).await?;

    Ok(ApiResponse::Ok {
        message: "Password reset successfully. You can now log in with your new password"
            .to_string(),
        data: None,
    })
}
