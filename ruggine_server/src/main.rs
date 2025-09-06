use std::sync::Arc;
use axum::http::Method;
use crate::config::{database, parameter};
use crate::config::database::DatabaseTrait;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing;
use tracing::error;
use crate::service::cpu_usage_log_service::CpuUsageLogServiceTrait;
use crate::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
use crate::service::cpu_usage_log_service::CpuUsageLogService;
use crate::utils::service_initializer::ServiceInitializer;

mod config;
mod docs;
mod routes;
mod dto;
mod error;
mod response;
mod entity;
mod repository;
mod state;
mod service;
mod middleware;
mod handler;
mod factory;
mod model;
mod utils;
mod websocket;

#[tokio::main]
async fn main() {
    // Initialize logging first
    tracing_subscriber::fmt::init();
    
    parameter::init();

    let database_url = parameter::get("DATABASE_URL");
    let connection = database::Database::init(database_url)
        .await
        .unwrap_or_else(|e| panic!("Database error: {}", e.to_string()));

    let host = format!("0.0.0.0:{}", parameter::get("PORT"));
    tracing::info!("🚀 Server is running on {}", host);
    
    let listener = TcpListener::bind(&host).await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", host, e));

    let connection_arc = Arc::new(connection);

    let app = routes::root::routes(Arc::clone(&connection_arc))
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // ⚠️ per sviluppo, in produzione specifica l'origin
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS, Method::PATCH])
                .allow_headers(Any)
        );

    let mut cpu_usage_log_service = CpuUsageLogService::new(
        Arc::new(CpuUsageLogRepository::new(&connection_arc))
    );
    cpu_usage_log_service.set_monitoring_interval_ms(
        parameter::get("LOG_IN_MILLISECONDS")
            .parse::<u64>()
            .expect("Invalid LOG_IN_MILLISECONDS")
    );
    match cpu_usage_log_service.start_monitoring().await {
        Ok(_) => {
            axum::serve(listener, app)
                .await
                .unwrap_or_else(|e| panic!("Server error: {}", e.to_string()));
        },
        Err(e) => {
            error!("Cannot start CPU usage log service error: {}. Stopping the server...", e);
            return;
        }
    }


}
