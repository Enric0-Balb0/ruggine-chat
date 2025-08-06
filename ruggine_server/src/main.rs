use std::sync::Arc;
use axum::http::Method;
use crate::config::{database, parameter};
use crate::config::database::DatabaseTrait;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing;

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

#[tokio::main]
async fn main() {
    // Initialize logging first
    tracing_subscriber::fmt::init();
    
    parameter::init();

    let database_url = parameter::get("DATABASE_URL");
    let connection = database::Database::init(database_url)
        .await
        .unwrap_or_else(|e| panic!("Database error: {}", e.to_string()));

    let host = format!("127.0.0.1:{}", parameter::get("PORT"));
    tracing::info!("🚀 Server is running on {}", host);
    
    let listener = TcpListener::bind(&host).await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", host, e));
    
    let app = routes::root::routes(Arc::new(connection))
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // ⚠️ per sviluppo, in produzione specifica l'origin
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
                .allow_headers(Any)
        );

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("Server error: {}", e.to_string()));
}
