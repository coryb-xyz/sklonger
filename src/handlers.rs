use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn health_live() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn health_ready() -> impl IntoResponse {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use tower::ServiceExt;

    fn app() -> Router {
        Router::new()
            .route("/health/live", get(health_live))
            .route("/health/ready", get(health_ready))
    }

    #[tokio::test]
    async fn test_health_live_returns_ok() {
        let response = app()
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

    #[tokio::test]
    async fn test_health_ready_returns_ok() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .expect("request should build"),
            )
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
