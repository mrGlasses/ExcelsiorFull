use crate::handlers::simple_handler::*;
use crate::{handlers::db_handler::*, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};
use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing::Span;

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/users", get(get_users))
        .route("/users", post(create_user))
        .route("/ping", get(get_pong))
        .route("/its-a-rainy-day", get(call_external_service))
        .route("/protected-enter", get(protected_route))
        .route("/params/:param_1/another_p/:param_2", get(get_params)) // localhost/params/1/another_p/textTest
        .route("/question_separator", get(get_question)) // localhost/question_separator?name=Jack&age=25&active=true
        .route("/body-data", post(post_body_data))
        .layer((
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Micros),
                )
                .on_failure(
                    |failure_class: ServerErrorsFailureClass, latency: Duration, _: &Span| {
                        tracing::error!(
                            failure_class = ?failure_class,
                            latency = ?latency,
                            "request failed"
                        );
                    },
                ),
            CompressionLayer::new(),
            RequestBodyLimitLayer::new(1024 * 1024 * 10), // 10MB limit
            TimeoutLayer::new(Duration::from_secs(60)),
        ))
        .with_state(state)
}

//more than 1 route file? search for "axum merge routes"
