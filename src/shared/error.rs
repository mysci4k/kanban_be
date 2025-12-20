use actix_web::{
    HttpResponse, ResponseError,
    body::BoxBody,
    http::{StatusCode, header::ContentType},
};
use derive_more::{Display, Error};
use sea_orm::DbErr;
use serde::Serialize;
use utoipa::ToSchema;
use validator::ValidationErrors;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationErrorSchema {
    status_code: u16,
    error: String,
    message: String,
}

#[derive(Debug, Display, Error)]
pub enum ApplicationError {
    #[display("Bad Request")]
    BadRequest { message: String },
    #[display("Unauthorized")]
    Unauthorized { message: String },
    #[display("Forbidden")]
    Forbidden { message: String },
    #[display("Not Found")]
    NotFound { message: String },
    #[display("Conflict")]
    Conflict { message: String },
    #[display("Too Many Requests")]
    TooManyRequests { message: String },
    #[display("Internal Server Error")]
    InternalError { message: String },
    #[display("Validation Error")]
    ValidationError { message: ValidationErrors },
    #[display("Database Error")]
    DatabaseError(DbErr),
}

impl From<ValidationErrors> for ApplicationError {
    fn from(err: ValidationErrors) -> Self {
        ApplicationError::ValidationError { message: err }
    }
}

impl From<DbErr> for ApplicationError {
    fn from(err: DbErr) -> Self {
        ApplicationError::DatabaseError(err)
    }
}

impl ApplicationError {
    fn message(&self) -> String {
        match self {
            ApplicationError::BadRequest { message } => message.to_owned(),
            ApplicationError::Unauthorized { message } => message.to_owned(),
            ApplicationError::Forbidden { message } => message.to_owned(),
            ApplicationError::NotFound { message } => message.to_owned(),
            ApplicationError::Conflict { message } => message.to_owned(),
            ApplicationError::TooManyRequests { message } => message.to_owned(),
            ApplicationError::InternalError { message } => message.to_owned(),
            ApplicationError::ValidationError { message } => message.to_string(),
            ApplicationError::DatabaseError(err) => err.to_string(),
        }
    }
}

impl ResponseError for ApplicationError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ApplicationError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApplicationError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            ApplicationError::Forbidden { .. } => StatusCode::FORBIDDEN,
            ApplicationError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApplicationError::Conflict { .. } => StatusCode::CONFLICT,
            ApplicationError::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
            ApplicationError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            ApplicationError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let response_body = ApplicationErrorSchema {
            message: self.message(),
            status_code: self.status_code().as_u16(),
            error: self.to_string(),
        };

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(response_body)
    }
}
