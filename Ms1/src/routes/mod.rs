use axum::{Router, routing::{get, post}};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse, DefaultOnRequest};
use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tracing::Span;
use tracing::Level;
use crate::{handlers::db_handler, state::AppState};
use crate::handlers::simple_handler;

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/users", get(db_handler::get_users))
        .route("/users", post(db_handler::create_user))
        .route("/ping", get(simple_handler::get_pong))
        .route("/its-a-rainy-day", get(simple_handler::call_external_service))
        .route("/protected-enter", get(simple_handler::protected_route))
        .route("/params/:param_1/another_p/:param_2", get(simple_handler::get_params)) // localhost/params/1/another_p/textTest
        .route("/question_separator", get(simple_handler::get_question)) // localhost/question_separator?name=Jack&age=25&active=true
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(tower_http::LatencyUnit::Micros)
                )
                .on_failure(|failure_class: ServerErrorsFailureClass, latency: Duration, _: &Span| {
                    tracing::error!(
                        failure_class = ?failure_class,
                        latency = ?latency,
                        "request failed"
                    );
                })
        )
        .with_state(state)
}

//more than 1 route file? search for "axum merge routes"