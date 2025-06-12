use crate::domain::general::{FilterParams, Message, Params};
use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use tracing::{error, info, warn};

pub async fn get_pong() -> Response {
    info!("PONG!");
    (StatusCode::OK, "PONG!").into_response()
}

pub async fn call_external_service() -> Response {
    info!("call_external_service called");
    let base_url = std::env::var("EXTERNAL_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:3001".to_string());

    let url = format!("{}/pong", base_url);
    let response = reqwest::get(url).await;
    info!("another microservice called");

    match response {
        Ok(resp) => {
            warn!("Ok response from external service");
            let json: Message = resp.json().await.unwrap_or(Message {
                code: 400,
                message_text: "Failed to parse external response".into(),
            });
            (
                StatusCode::from_u16(json.code as u16).unwrap_or(StatusCode::EXPECTATION_FAILED),
                format!("{} !!", json.message_text),
            )
                .into_response()
        }
        Err(err) => {
            error!("Error response from external service: {}", err);
            (
                StatusCode::EXPECTATION_FAILED,
                "Failed to reach external service",
            )
                .into_response()
        }
    }
}

pub async fn protected_route(headers: HeaderMap) -> Response {
    // Verifies if the specific header exists and have the correct value
    match headers.get("X-Custom-Header") {
        Some(header_value) if header_value == "secret-value" => {
            (StatusCode::OK, "Access Granted!").into_response()
        }
        _ => (StatusCode::UNAUTHORIZED, "Invalid or missing header").into_response(),
    }
}

pub async fn get_params(Path(Params { param_1, param_2 }): Path<Params>) -> Response {
    (
        StatusCode::OK,
        format!("Parameter 1: {}, Parameter 2: {}", param_1, param_2),
    )
        .into_response()
}

pub async fn get_question(Query(params): Query<FilterParams>) -> Response {
    let response = format!(
        "Filters: name={}, age={}, active={}",
        params.name.unwrap_or_default(),
        params.age.unwrap_or_default(),
        params.active.unwrap_or_default()
    );
    (StatusCode::OK, response).into_response()
}

pub async fn post_body_data(Json(payload): Json<Message>) -> Response {
    info!("Received payload: {:?}", payload);
    let response = format!(
        "Received message with code: {}, text: {}",
        payload.code, payload.message_text
    );
    (StatusCode::OK, response).into_response()
}
