use axum::extract::State;
use axum::Json;
use crate::state::AppState;
use crate::handlers::simple_handler::Message;

pub async fn get_answer2(State(_state): State<AppState>) -> Json<Message> {
    let mut msg = Message::default();
    msg.code = 200;
    msg.message_text = "E-E!".to_string();
    Json(msg)
}