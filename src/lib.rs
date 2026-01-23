pub mod bluesky;
pub mod config;
pub mod error;
pub mod handlers;
pub mod html;
pub mod logging;

use std::time::Duration;

use axum::{routing::get, Router};

use crate::bluesky::BlueskyClient;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub client: BlueskyClient,
}

pub fn create_app(config: &Config) -> anyhow::Result<Router> {
    let client = BlueskyClient::new(
        &config.bluesky_api_url,
        Duration::from_secs(config.request_timeout_seconds),
    )?;

    let state = AppState { client };

    Ok(Router::new()
        .route("/", get(handlers::get_thread))
        .route("/thread", get(handlers::get_thread))
        // Use streaming handler for direct path access (most common use case)
        .route(
            "/profile/{handle}/post/{post_id}",
            get(handlers::get_thread_streaming),
        )
        .route("/health/live", get(handlers::health_live))
        .route("/health/ready", get(handlers::health_ready))
        .with_state(state))
}
