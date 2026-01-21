pub mod config;
pub mod handlers;
pub mod logging;

use axum::{routing::get, Router};

pub fn create_app() -> Router {
    Router::new()
        .route("/health/live", get(handlers::health_live))
        .route("/health/ready", get(handlers::health_ready))
}
