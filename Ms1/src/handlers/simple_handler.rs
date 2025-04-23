use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tracing::{info, warn, error};
use crate::domain::general::Message;
use crate::state::AppState;

pub async fn get_pong(State(_state): State<AppState>) -> Response {
    info!("PONG!");
    (
        StatusCode::OK, 
        "PONG!"
    ).into_response()
}

pub async fn call_external_service() -> Response {
    info!("call_external_service called");
    let url = "http://localhost:3001/pong";
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
                format!("{} !!", json.message_text)
            ).into_response()
        }
        Err(err) => {
            error!("Error response from external service: {}", err);
            (
                StatusCode::EXPECTATION_FAILED, 
                "Failed to reach external service",
                ).into_response()
        } 
    }
}
