use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header::USER_AGENT, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::{info, warn};

use crate::bluesky::client::ClientError;
use crate::bluesky::parse_bluesky_url;
use crate::bluesky::types::StreamEvent;
use crate::error::AppError;
use crate::html::{
    landing_page, render_post, render_thread, streaming_error, streaming_footer, streaming_head,
    streaming_loading_indicator, streaming_post_before_indicator, PollingConfig,
    StreamingHeadOptions,
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

#[derive(Deserialize)]
pub struct ThreadUpdatesQuery {
    pub handle: String,
    pub post_id: String,
    pub since_cid: String,
}

/// Common social media crawler User-Agent patterns.
/// These crawlers fetch pages to generate link previews.
const SOCIAL_CRAWLER_PATTERNS: &[&str] = &[
    "Twitterbot",
    "facebookexternalhit",
    "LinkedInBot",
    "WhatsApp",
    "Slackbot",
    "TelegramBot",
    "Discordbot",
    // Some crawlers for general link previews
    "Googlebot",
    "bingbot",
    "Applebot",
];

/// Check if a User-Agent string belongs to a social media crawler.
/// These crawlers are served the non-streaming path for proper Open Graph tags.
fn is_social_crawler(user_agent: &str) -> bool {
    SOCIAL_CRAWLER_PATTERNS
        .iter()
        .any(|pattern| user_agent.contains(pattern))
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

    let html = render_thread(&thread, &state.config.public_url);
    Ok(Html(html))
}

/// Handler for polling thread updates.
/// Returns new posts (if any) since the given CID as HTML fragments.
pub async fn get_thread_updates(
    State(state): State<AppState>,
    Query(params): Query<ThreadUpdatesQuery>,
) -> Result<Response, AppError> {
    let thread = state
        .client
        .get_thread_by_handle(&params.handle, &params.post_id)
        .await
        .map_err(map_client_error)?;

    // Capture last post timestamp before posts are consumed by into_iter
    let last_post_time = thread.posts.last().map(|p| p.created_at);

    // Find posts after the since_cid
    let since_idx = thread.posts.iter().position(|p| p.cid == params.since_cid);

    let new_posts: Vec<_> = match since_idx {
        Some(idx) => thread.posts.into_iter().skip(idx + 1).collect(),
        None => {
            // CID not found - return all posts (thread might have been restructured)
            thread.posts
        }
    };

    if new_posts.is_empty() {
        // Check if the thread has gone stale (last post older than threshold)
        let is_stale = last_post_time
            .map(|ts| {
                Utc::now().signed_duration_since(ts).num_seconds()
                    >= state.config.poll_disable_after as i64
            })
            .unwrap_or(true);

        return Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("X-Thread-Stale", if is_stale { "true" } else { "false" })
            .body(Body::empty())
            .unwrap());
    }

    // Safe to unwrap: we already checked that new_posts is not empty
    let last_cid = &new_posts.last().unwrap().cid;
    let post_count = new_posts.len();

    // Render posts as HTML fragments
    let html: String = new_posts
        .iter()
        .map(|post| render_post(post, &thread.author.handle))
        .collect();

    info!(
        handle = %params.handle,
        post_count = post_count,
        "returning thread updates"
    );

    // New posts arrived, so the thread is active
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("X-Last-CID", last_cid.as_str())
        .header("X-Post-Count", post_count.to_string())
        .header("X-Thread-Stale", "false")
        .body(Body::from(html))
        .unwrap())
}

/// Streaming handler that sends HTML progressively as posts are fetched.
/// This provides better perceived responsiveness for long threads.
///
/// For social media crawlers (detected via User-Agent), this handler returns
/// the non-streaming response with proper Open Graph meta tags for rich previews.
pub async fn get_thread_streaming(
    State(state): State<AppState>,
    Path(params): Path<ThreadPath>,
    headers: HeaderMap,
) -> Response {
    use futures::stream::StreamExt as _;
    use tokio::sync::mpsc;

    // Check if this is a social media crawler requesting link preview data.
    // Crawlers don't benefit from streaming and need the full HTML with OG tags.
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if is_social_crawler(user_agent) {
        info!(
            handle = %params.handle,
            post_id = %params.post_id,
            user_agent = %user_agent,
            "serving non-streaming response for social crawler"
        );
        return match fetch_and_render_thread(&state, &params.handle, &params.post_id).await {
            Ok(html) => html.into_response(),
            Err(e) => e.into_response(),
        };
    }

    info!(handle = %params.handle, post_id = %params.post_id, "fetching thread (streaming)");

    let (tx, rx) = mpsc::channel::<Result<String, std::convert::Infallible>>(16);

    let client = state.client.clone();
    let config = state.config.clone();
    let handle = params.handle.clone();
    let post_id = params.post_id.clone();

    tokio::spawn(async move {
        let mut author_handle = handle.clone();
        let mut first_post_id: Option<String> = None;
        let mut post_count = 0;
        let mut last_cid = String::new();
        let mut last_post_timestamp: Option<DateTime<Utc>> = None;

        let post_id_for_url = post_id.clone();
        let stream = client.get_thread_streaming(handle, post_id);
        futures::pin_mut!(stream);

        while let Some(event) = stream.next().await {
            let chunk = match event {
                Ok(StreamEvent::Header(author)) => {
                    author_handle = author.handle.clone();
                    let thread_url = format!(
                        "{}/profile/{}/post/{}",
                        config.public_url, author.handle, post_id_for_url
                    );
                    streaming_head(StreamingHeadOptions {
                        author_handle: &author.handle,
                        author_display_name: author.display_name.as_deref(),
                        avatar_url: author.avatar_url.as_deref(),
                        profile_url: &author.profile_url(),
                        lang: None,
                        first_post_text: None, // Not available in streaming mode
                        thread_url: &thread_url,
                    })
                }
                Ok(StreamEvent::Post(post)) => {
                    if first_post_id.is_none() {
                        first_post_id = post.uri.rsplit('/').next().map(String::from);
                    }
                    last_cid = post.cid.clone();
                    last_post_timestamp = Some(post.created_at);

                    let post_html = render_post(&post, &author_handle);
                    post_count += 1;

                    if post_count == 1 {
                        format!("{}{}", post_html, streaming_loading_indicator())
                    } else {
                        streaming_post_before_indicator(&post_html)
                    }
                }
                Ok(StreamEvent::Done) => {
                    let post_id_str = first_post_id.as_deref().unwrap_or("");
                    let original_url = format!(
                        "https://bsky.app/profile/{}/post/{}",
                        author_handle, post_id_str
                    );

                    // Enable polling if configured; mark stale threads so
                    // auto-refresh starts off but can be toggled on by the user.
                    let is_thread_recent = last_post_timestamp
                        .map(|ts| {
                            let age = Utc::now().signed_duration_since(ts).num_seconds();
                            age < config.poll_disable_after as i64
                        })
                        .unwrap_or(false);

                    let polling_config = if config.poll_enabled {
                        Some(PollingConfig {
                            handle: author_handle.clone(),
                            post_id: post_id_str.to_string(),
                            last_cid: last_cid.clone(),
                            initial_interval: config.poll_initial_interval,
                            max_interval: config.poll_max_interval,
                            disable_after: config.poll_disable_after,
                            stale: !is_thread_recent,
                            last_post_iso: last_post_timestamp
                                .map(|ts| ts.to_rfc3339())
                                .unwrap_or_default(),
                        })
                    } else {
                        None
                    };

                    streaming_footer(&original_url, polling_config.as_ref())
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

// PWA handlers

/// Serve the PWA manifest for Android Web Share Target support.
pub async fn manifest() -> impl IntoResponse {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/manifest+json",
        )],
        crate::pwa::MANIFEST_JSON,
    )
}

/// Serve the service worker for PWA installability.
pub async fn service_worker() -> impl IntoResponse {
    (
        [
            (axum::http::header::CONTENT_TYPE, "application/javascript"),
            (
                axum::http::header::HeaderName::from_static("service-worker-allowed"),
                "/",
            ),
        ],
        crate::pwa::SERVICE_WORKER_JS,
    )
}

/// Serve the PWA icon.
pub async fn icon() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
        crate::pwa::APP_ICON_SVG,
    )
}

#[derive(Deserialize)]
pub struct ShareQuery {
    pub url: Option<String>,
    pub text: Option<String>,
    #[allow(dead_code)]
    pub title: Option<String>,
}

/// Handle Web Share Target API requests.
/// Extracts Bluesky URL from shared content and redirects to thread view.
pub async fn share_target(Query(params): Query<ShareQuery>) -> Result<Redirect, AppError> {
    let url = extract_bluesky_url(&params)?;
    info!(url = %url, "share target received Bluesky URL");

    let encoded_url = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("url", &url)
        .finish();
    let redirect_url = format!("/?{}", encoded_url);

    Ok(Redirect::to(&redirect_url))
}

/// Extract a Bluesky URL from share target parameters.
/// Checks the url param first, then searches the text param for bsky.app URLs.
fn extract_bluesky_url(params: &ShareQuery) -> Result<String, AppError> {
    // Check url param first
    if let Some(url) = &params.url {
        if url.contains("bsky.app") {
            return Ok(url.clone());
        }
    }

    // Check text param for bsky.app URL
    if let Some(text) = &params.text {
        if let Some(url) = find_bluesky_url_in_text(text) {
            return Ok(url);
        }
    }

    Err(AppError::BadRequest(
        "No Bluesky URL found in shared content".to_string(),
    ))
}

/// Find a bsky.app URL in text using regex.
fn find_bluesky_url_in_text(text: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"https?://bsky\.app/profile/[^\s]+/post/[^\s]+").ok()?;
    re.find(text).map(|m| m.as_str().to_string())
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

    #[test]
    fn test_extract_bluesky_url_from_url_param() {
        let params = ShareQuery {
            url: Some("https://bsky.app/profile/user.bsky.social/post/abc123".to_string()),
            text: None,
            title: None,
        };
        let result = extract_bluesky_url(&params);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "https://bsky.app/profile/user.bsky.social/post/abc123"
        );
    }

    #[test]
    fn test_extract_bluesky_url_from_text() {
        let params = ShareQuery {
            url: None,
            text: Some(
                "Check this out: https://bsky.app/profile/user.bsky.social/post/abc123 cool!"
                    .to_string(),
            ),
            title: None,
        };
        let result = extract_bluesky_url(&params);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "https://bsky.app/profile/user.bsky.social/post/abc123"
        );
    }

    #[test]
    fn test_extract_bluesky_url_not_found() {
        let params = ShareQuery {
            url: Some("https://twitter.com/user/status/123".to_string()),
            text: None,
            title: None,
        };
        let result = extract_bluesky_url(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_bluesky_url_prefers_url_param() {
        let params = ShareQuery {
            url: Some("https://bsky.app/profile/first.bsky.social/post/111".to_string()),
            text: Some(
                "Other link: https://bsky.app/profile/second.bsky.social/post/222".to_string(),
            ),
            title: None,
        };
        let result = extract_bluesky_url(&params);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("first.bsky.social"));
    }

    #[test]
    fn test_find_bluesky_url_in_text() {
        let text = "Look at this https://bsky.app/profile/test.bsky.social/post/xyz and more";
        let result = find_bluesky_url_in_text(text);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "https://bsky.app/profile/test.bsky.social/post/xyz"
        );
    }

    #[test]
    fn test_find_bluesky_url_in_text_no_match() {
        let text = "No Bluesky links here, just https://example.com";
        let result = find_bluesky_url_in_text(text);
        assert!(result.is_none());
    }
}
