use std::sync::Arc;
use crate::config::{database, parameter};
use crate::config::database::DatabaseTrait;
use tokio::net::TcpListener;

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

#[tokio::main]
async fn main() {
    parameter::init();

    let database_url = parameter::get("DATABASE_URL");
    let connection = database::Database::init(database_url)
        .await
        .unwrap_or_else(|e| panic!("Database error: {}", e.to_string()));

    let host = format!("127.0.0.1:{}", parameter::get("PORT"));
    println!("🚀 Server is running on {}", host);
    tracing_subscriber::fmt::init();
    
    let listener = TcpListener::bind(&host).await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", host, e));
    
    axum::serve(listener, routes::root::routes(Arc::new(connection)))
        .await
        .unwrap_or_else(|e| panic!("Server error: {}", e.to_string()));
}
