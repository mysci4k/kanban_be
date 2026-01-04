use crate::{
    presentation::{
        configure_auth_roures, configure_board_routes, configure_column_routes,
        configure_task_routes, configure_user_routes, configure_websocket_routes, http::ApiDoc,
        middleware::RequireAuth,
    },
    shared::{
        config::{AppState, CustomRootSpanBuilder},
        utils::constants::{BASE_URL, ENABLED_SCALAR, REDIS_URL, SESSION_KEY},
    },
};
use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::RedisSessionStore};
use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    cookie::{Key, SameSite, time::Duration},
    get,
    http::header,
    web,
};
use std::io::Result;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

#[get("/")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Server is up!")
}

pub async fn configure_server(
    app_state: AppState,
    server_address: &str,
    server_port: u16,
) -> Result<actix_web::dev::Server> {
    let redis_store = RedisSessionStore::new(REDIS_URL.as_str())
        .await
        .expect("Failed to connect to Redis for session storage");

    let session_key_bytes = SESSION_KEY.as_bytes();
    if session_key_bytes.len() < 64 {
        panic!("SESSION_KEY must be at least 64 bytes long");
    }

    let session_key = Key::from(session_key_bytes);

    let openapi = ApiDoc::openapi();

    let server = HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(app_state.auth_service.clone()))
            .app_data(web::Data::new(app_state.user_service.clone()))
            .app_data(web::Data::new(app_state.board_service.clone()))
            .app_data(web::Data::new(app_state.column_service.clone()))
            .app_data(web::Data::new(app_state.task_service.clone()))
            .app_data(web::Data::new(app_state.websocket_service.clone()))
            .wrap(
                Cors::default()
                    .allowed_origin(&BASE_URL)
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                    ])
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(TracingLogger::<CustomRootSpanBuilder>::new())
            .wrap(RequireAuth)
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), session_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(1)))
                    .cookie_name("user-session".to_string())
                    .cookie_same_site(SameSite::Lax)
                    .cookie_http_only(true)
                    .cookie_secure(true)
                    .build(),
            );

        if *ENABLED_SCALAR {
            app = app.service(Scalar::with_url("/scalar", openapi.clone()));
        }

        app.service(
            web::scope("/api")
                .service(health_check)
                .configure(configure_auth_roures)
                .configure(configure_user_routes)
                .configure(configure_board_routes)
                .configure(configure_column_routes)
                .configure(configure_task_routes)
                .configure(configure_websocket_routes),
        )
    })
    .bind((server_address, server_port))?;

    Ok(server.run())
}
