// The small demos: ping, calling another service, headers, path/query/body extraction.
use crate::handlers::simple_handler::{
    call_external_service, get_params, get_pong, get_question, post_body_data, protected_route,
};
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ping", get(get_pong))
        .route("/its-a-rainy-day", get(call_external_service))
        .route("/protected-enter", get(protected_route))
        .route("/params/{param_1}/another_p/{param_2}", get(get_params)) // localhost/params/1/another_p/textTest
        .route("/question_separator", get(get_question)) // localhost/question_separator?name=Jack&age=25&active=true
        .route("/body-data", post(post_body_data))
}
