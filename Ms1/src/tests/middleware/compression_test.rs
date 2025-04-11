use axum::{body::Body, http::{Request, StatusCode}, response::IntoResponse};
use tower::ServiceExt;
use crate::tests::test_utils;

async fn handler(body: String) -> impl IntoResponse {
    format!("Echo: {}", body)
}

#[tokio::test]
async fn test_compression_header() {
    let app = test_utils::app_with_middleware(handler);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Accept-Encoding", "gzip")
                .method("POST")
                .body(Body::from("hello gzip"))
                .unwrap(),
        )
        .await
        .unwrap();

    let encoding = response.headers().get("Content-Encoding");
    assert!(encoding.is_some(), "Expected Content-Encoding header");
    assert_eq!(encoding.unwrap(), "gzip");
    assert_eq!(response.status(), StatusCode::OK);
}
