use crate::{
    presentation::{
        configure_auth_roures, configure_board_routes, configure_column_routes,
        configure_task_routes, configure_user_routes, configure_websocket_routes, http::ApiDoc,
        middleware::RequireAuth,
    },
    shared::{
        config::AppState,
        utils::constants::{REDIS_URL, SESSION_KEY},
    },
};
use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::RedisSessionStore};
use actix_web::{
    App, HttpResponse, HttpServer, Responder,
    cookie::{Key, time::Duration},
    get,
    middleware::Logger,
    web,
};
use std::io::Result;
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

    let session_key = Key::from(SESSION_KEY.as_bytes());

    let openapi = ApiDoc::openapi();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.auth_service.clone()))
            .app_data(web::Data::new(app_state.user_service.clone()))
            .app_data(web::Data::new(app_state.board_service.clone()))
            .app_data(web::Data::new(app_state.column_service.clone()))
            .app_data(web::Data::new(app_state.task_service.clone()))
            .app_data(web::Data::new(app_state.websocket_service.clone()))
            .wrap(Logger::default())
            .wrap(RequireAuth)
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), session_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::days(1)))
                    .cookie_name("user-session".to_string())
                    .build(),
            )
            .service(Scalar::with_url("/scalar", openapi.clone()))
            .service(
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
