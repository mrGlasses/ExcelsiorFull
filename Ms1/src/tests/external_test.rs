#[cfg(test)]
mod tests {
    use axum::{body::Body, http::{Request, StatusCode}, Router};
    use tower::ServiceExt;
    use crate::handlers::user_handler::ExternalResponse;

    #[tokio::test]
    async fn test_call_external_service_mock() {
        let app = Router::new().route("/itsaRainyDay", axum::routing::get(|| async {
            axum::Json(ExternalResponse { message: "mocked response".to_string() })
        }));

        let response = app.oneshot(
            Request::builder()
                .uri("/itsaRainyDay")
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
