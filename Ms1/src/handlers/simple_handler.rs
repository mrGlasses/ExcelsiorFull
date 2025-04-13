use axum::extract::State;
use axum::Json;
use crate::state::AppState;
use crate::models::general::Message;


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