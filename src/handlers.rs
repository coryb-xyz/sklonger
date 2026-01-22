use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use tracing::{info, warn};

use crate::bluesky::client::ClientError;
use crate::bluesky::parse_bluesky_url;
use crate::error::AppError;
use crate::html::{landing_page, render_thread};
use crate::AppState;

#[derive(Deserialize)]
pub struct ThreadQuery {
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub struct ThreadPath {
    pub handle: String,
    pub post_id: String,
}

fn map_client_error(e: ClientError) -> AppError {
    warn!(error = %e, "failed to fetch thread");
    match &e {
        ClientError::NotFound => AppError::NotFound("post not found or deleted".to_string()),
        ClientError::Blocked => AppError::NotFound("post is blocked".to_string()),
        ClientError::RateLimited => AppError::RateLimited,
        ClientError::Http(err) if err.is_connect() => {
            AppError::ServiceUnavailable("cannot reach Bluesky API".to_string())
        }
        ClientError::Http(err) if err.is_timeout() => {
            AppError::ServiceUnavailable("request timed out".to_string())
        }
        _ => AppError::Internal(e.into()),
    }
}

pub async fn get_thread(
    State(state): State<AppState>,
    Query(params): Query<ThreadQuery>,
) -> Result<Html<String>, AppError> {
    let url = match params.url {
        Some(u) if !u.is_empty() => u,
        _ => return Ok(Html(landing_page())),
    };

    info!(url = %url, "fetching thread");

    let parsed = parse_bluesky_url(&url).map_err(|e| AppError::BadRequest(e.to_string()))?;

    fetch_and_render_thread(&state, &parsed.handle, &parsed.post_id).await
}

pub async fn get_thread_by_path(
    State(state): State<AppState>,
    Path(params): Path<ThreadPath>,
) -> Result<Html<String>, AppError> {
    info!(handle = %params.handle, post_id = %params.post_id, "fetching thread by path");

    fetch_and_render_thread(&state, &params.handle, &params.post_id).await
}

async fn fetch_and_render_thread(
    state: &AppState,
    handle: &str,
    post_id: &str,
) -> Result<Html<String>, AppError> {
    let thread = state
        .client
        .get_thread_by_handle(handle, post_id)
        .await
        .map_err(map_client_error)?;

    info!(
        author = %thread.author.handle,
        post_count = thread.posts.len(),
        "thread fetched successfully"
    );

    let html = render_thread(&thread);
    Ok(Html(html))
}

pub async fn health_live() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn health_ready(State(state): State<AppState>) -> impl IntoResponse {
    match state.client.resolve_handle("bsky.app").await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use tower::ServiceExt;

    fn health_app() -> Router {
        Router::new().route("/health/live", get(health_live))
    }

    #[tokio::test]
    async fn test_health_live_returns_ok() {
        let response = health_app()
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
