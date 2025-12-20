use actix_web::{
    HttpRequest, HttpResponse, Responder,
    body::BoxBody,
    http::{StatusCode, header::ContentType},
};
use derive_more::Display;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponseSchema<T> {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rows_affected: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_pages: Option<u64>,
}

#[derive(Debug, Display)]
pub enum ApiResponse<T> {
    #[display("{}", message)]
    Created { message: String, data: T },
    #[display("{}", message)]
    Updated { message: String, data: T },
    #[display("{}", message)]
    Deleted { message: String, rows_affected: u64 },
    #[display("{}", message)]
    Found {
        message: String,
        data: T,
        page: Option<u64>,
        total_pages: Option<u64>,
    },
    #[display("{}", message)]
    Ok { message: String, data: Option<T> },
}

impl<T> ApiResponse<T> {
    fn rows_affected(&self) -> Option<u64> {
        match self {
            ApiResponse::Deleted { rows_affected, .. } => Some(*rows_affected),
            _ => None,
        }
    }

    fn data(&self) -> Option<&T> {
        match self {
            ApiResponse::Created { data, .. } => Some(data),
            ApiResponse::Updated { data, .. } => Some(data),
            ApiResponse::Found { data, .. } => Some(data),
            ApiResponse::Ok { data, .. } => data.as_ref(),
            _ => None,
        }
    }

    fn page_info(&self) -> (Option<u64>, Option<u64>) {
        match self {
            ApiResponse::Found {
                page, total_pages, ..
            } => (*page, *total_pages),
            _ => (None, None),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiResponse::Created { .. } => StatusCode::CREATED,
            ApiResponse::Updated { .. } => StatusCode::OK,
            ApiResponse::Deleted { .. } => StatusCode::OK,
            ApiResponse::Found { .. } => StatusCode::OK,
            ApiResponse::Ok { .. } => StatusCode::OK,
        }
    }
}

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let (page, total_pages) = self.page_info();
        let response_body = ApiResponseSchema {
            message: self.to_string(),
            data: self.data(),
            rows_affected: self.rows_affected(),
            page,
            total_pages,
        };

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(response_body)
    }
}
