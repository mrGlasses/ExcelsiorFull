use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::state::AppState;


#[derive(Serialize, Deserialize, FromRow)]
pub struct Message {
    pub code: i32,
    pub message_text: String,
}

impl Default for Message {
    fn default() -> Message {
        Message {
            code: Default::default(),
            message_text: Default::default(),
        }
    }
}

pub async fn get_pong(State(_state): State<AppState>) -> Json<Message> {
    let mut msg= Message::default();
    msg.code = 200;
    msg.message_text = "PONG!".to_string();
    Json(msg)
}

pub async fn call_external_service() -> Json<Message> {
    let url = "http://localhost:3001/pong";                      
    let response = reqwest::get(url).await;                      

    match response {                                             
        Ok(resp) => {                                           
            let json: Message = resp.json().await.unwrap_or(Message {
                code: 400,
                message_text: "Failed to parse external response".into(),
            });
            let new_message = Message{
                code: json.code,
                message_text: json.message_text + "!!"
            };
            Json(new_message)
        },                                                                              
        Err(_) => Json(Message {
            code: 400,
            message_text: "Failed to reach external service".into(),
        })                                                                              
    }                                                                                   
}  