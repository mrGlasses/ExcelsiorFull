use axum::extract::State;
use axum::Json;
use crate::state::AppState;

pub async fn get_answer2(State(_state): State<AppState>) -> Json<crate::handlers::simple_handler::Message> {
    let mut msg= crate::handlers::simple_handler::Message::default();
    msg.code = 200;
    msg.message_text = "E-E!".to_string();
    Json(msg)
}