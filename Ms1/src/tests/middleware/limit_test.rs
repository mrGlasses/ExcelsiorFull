use axum::{body::Body, http::{Request, StatusCode}, response::IntoResponse};
use tower::ServiceExt;
use crate::tests::test_utils;

async fn handler(body: String) -> impl IntoResponse {
    format!("Echo: {}", body)
}

#[tokio::test]
async fn test_limit_exceeded() {
    let app = test_utils::app_with_middleware(handler);

    let long_body = "A".repeat(1024); // exceeds 16 bytes
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .method("POST")
                .body(Body::from(long_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
