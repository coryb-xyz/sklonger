use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use tracing::{info, warn};

use crate::bluesky::client::ClientError;
use crate::bluesky::parse_bluesky_url;
use crate::bluesky::types::StreamEvent;
use crate::error::AppError;
use crate::html::{
    landing_page, render_post, render_thread, streaming_error, streaming_footer, streaming_head,
    streaming_loading_indicator, streaming_post_before_indicator, StreamingHeadOptions,
};
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

/// Streaming handler that sends HTML progressively as posts are fetched.
/// This provides better perceived responsiveness for long threads.
pub async fn get_thread_streaming(
    State(state): State<AppState>,
    Path(params): Path<ThreadPath>,
) -> Response {
    use futures::stream::StreamExt as _;
    use tokio::sync::mpsc;

    info!(handle = %params.handle, post_id = %params.post_id, "fetching thread (streaming)");

    let (tx, rx) = mpsc::channel::<Result<String, std::convert::Infallible>>(16);

    let client = state.client.clone();
    let handle = params.handle.clone();
    let post_id = params.post_id.clone();

    tokio::spawn(async move {
        let mut author_handle = handle.clone();
        let mut first_post_uri: Option<String> = None;
        let mut post_count = 0;

        let stream = client.get_thread_streaming(handle, post_id);
        futures::pin_mut!(stream);

        while let Some(event) = stream.next().await {
            let chunk = match event {
                Ok(StreamEvent::Header(author)) => {
                    author_handle = author.handle.clone();
                    streaming_head(StreamingHeadOptions {
                        author_handle: &author.handle,
                        author_display_name: author.display_name.as_deref(),
                        avatar_url: author.avatar_url.as_deref(),
                        profile_url: &author.profile_url(),
                        lang: None,
                    })
                }
                Ok(StreamEvent::Post(post)) => {
                    if first_post_uri.is_none() {
                        first_post_uri = Some(post.uri.clone());
                    }
                    let post_html = render_post(&post, &author_handle);
                    post_count += 1;

                    if post_count == 1 {
                        format!("{}{}", post_html, streaming_loading_indicator())
                    } else {
                        streaming_post_before_indicator(&post_html)
                    }
                }
                Ok(StreamEvent::Done) => {
                    let post_id_str = first_post_uri
                        .as_deref()
                        .and_then(|uri| uri.rsplit('/').next())
                        .unwrap_or("");
                    let original_url = format!(
                        "https://bsky.app/profile/{}/post/{}",
                        author_handle, post_id_str
                    );
                    streaming_footer(&original_url)
                }
                Err(e) => {
                    warn!(error = %e, "streaming error");
                    streaming_error(&e.to_string())
                }
            };

            if tx.send(Ok(chunk)).await.is_err() {
                break;
            }
        }
    });

    let body_stream = tokio_stream::wrappers::ReceiverStream::new(rx);

    Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Transfer-Encoding", "chunked")
        .header("X-Content-Type-Options", "nosniff")
        .body(Body::from_stream(body_stream))
        .unwrap()
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
