use actix_web::{
    Error, HttpMessage,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
};
use tracing_actix_web::{DefaultRootSpanBuilder, RequestId, RootSpanBuilder};

pub struct CustomRootSpanBuilder;

impl RootSpanBuilder for CustomRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> tracing::Span {
        let request_id = request
            .extensions()
            .get::<RequestId>()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        tracing::info_span!(
            "http_request",
            request_id = %request_id,
            http.method = %request.method(),
            http.route = %request.path(),
            http.status_code = tracing::field::Empty,
            http.host = %request
                .headers()
                .get("Host")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown"),
            http.client_ip = %request
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("unknown"),
            http.user_agent = %request.headers()
                .get("User-Agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown"),
            otel.kind = "server",
            otel.status_code = tracing::field::Empty,
        )
    }

    fn on_request_end<B: MessageBody>(
        span: tracing::Span,
        outcome: &Result<ServiceResponse<B>, Error>,
    ) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}
