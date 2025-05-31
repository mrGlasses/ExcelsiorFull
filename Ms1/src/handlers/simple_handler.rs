use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::{info, warn, error};
use crate::domain::general::{FilterParams, Message, Params};

pub async fn get_pong() -> Response {
    info!("PONG!");
    (
        StatusCode::OK, 
        "PONG!"
    ).into_response()
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

pub async fn protected_route(headers: HeaderMap) -> Response {
    // Verifies if the specific header exists and have the correct value
    match headers.get("X-Custom-Header") {
        Some(header_value) if header_value == "secret-value" => {
            (StatusCode::OK, "Access Granted!").into_response()
        }
        _ => (StatusCode::UNAUTHORIZED, "Invalid or missing header").into_response()
    }
}

pub async fn get_params(
    Path(Params { param_1, param_2 }): Path<Params>,
) -> Response {
    (
        StatusCode::OK,
        format!("Parameter 1: {}, Parameter 2: {}", param_1, param_2)
    ).into_response()
}

pub async fn get_question(Query(params): Query<FilterParams>) -> Response {
    let response = format!(
        "Filters: name={}, age={}, active={}",
        params.name.unwrap_or_default(),
        params.age.unwrap_or_default(),
        params.active.unwrap_or_default()
    );
    (
        StatusCode::OK,
        response
    ).into_response()
}
// 
// /*################################################################################################
// 
//         ----------------------------------------TESTS-----------------------------------
// 
// #################################################################################################*/
// /*
//      BY CONVENTION
// 
//      We are obligated to write unit tests inside the same file as the normal code.
// 
//  */
// 
// #[cfg(test)]
// mod tests {
//     use axum::body::HttpBody;
//     use super::*;
//     use axum::http::StatusCode;
//     use httpmock::prelude::*;
// 
// 
//     #[tokio::test]
//     async fn test_get_pong() {
//         let response = get_pong().await;
//         let body = response.into_body().collect().await.unwrap().to_bytes();
//         
//         assert_eq!(&body[..], b"PONG!");
//     }
// 
//     #[tokio::test]
//     async fn test_call_external_service_ok() {
//         let server = MockServer::start_async().await;
// 
//         let hello_mock = server.mock_async(|when, then| {
//             when.method("GET")
//                 .path("/pong");
//             then.status(200)
//                 .header("content-type", "text/html; charset=UTF-8")
//                 .body(r#"{"code": 200, "message_text": "PONG"}"#);
//         }).await;
//         
//         std::env::set_var("EXTERNAL_SERVICE_URL", server.url(""));
//         
//         let response = call_external_service().await;
// 
//         hello_mock.assert();
// 
//         assert_eq!(response.into_response().status(), StatusCode::OK);
//     }
//     #[tokio::test]
//     async fn test_call_external_service_fail() {
//         // let server = MockServer::start_async().await;
//         // 
//         // let hello_mock = server.mock_async(|when, then| {
//         //     when.method("GET")
//         //         .path("/pong");
//         //     then.status(200)
//         //         .header("content-type", "text/html; charset=UTF-8")
//         //         .body(r#"{"code": 400, "message_text": "FAIL"}"#);
//         // }).await;
//         // 
//         
//         //forcing invalid link
//         std::env::set_var("EXTERNAL_SERVICE_URL", "localhost:99999" /*server.url("")*/);
//         
//         let response = call_external_service().await;
// 
//         // hello_mock.assert();
// 
//         assert_ne!(response.into_response().status(), StatusCode::OK);
//     }
// 
// 
//     #[tokio::test]
//     async fn test_protected_route_no_auth() {
//         let headers = HeaderMap::new();
//         let response = protected_route(headers).await;
// 
//         assert_eq!(response.into_response().status(), StatusCode::UNAUTHORIZED);
//     }
// 
//     #[tokio::test]
//     async fn test_protected_route_with_auth() {
//         let mut headers = HeaderMap::new();
//         headers.insert(
//             "X-Custom-Header",
//             axum::http::HeaderValue::from_str("secret-value").unwrap()
//         );
// 
//         let response = protected_route(headers).await;
//         assert_eq!(response.into_response().status(), StatusCode::OK);
//     }
// 
//     #[tokio::test]
//     async fn test_get_params() {
//         let param_in = Path(
//             Params{
//                 param_1: 1,
//                 param_2: "test2".to_string()
//             });
//         let response = get_params(param_in).await;
//         let body = response.into_body().collect().await.unwrap().to_bytes();
// 
//         assert_eq!(body, "Parameter 1: 1, Parameter 2: test2");
//     }
// 
//     #[tokio::test]
//     async fn test_get_question() {
//         let query = Query(
//             FilterParams{
//                 name: Option::from("Jack".to_string()),
//                 age: Option::from(25),
//                 active: Option::from(true)
//             }
//         );
// 
//         let response = get_question(query).await;
//         let body = response.into_body().collect().await.unwrap().to_bytes();
//         match std::str::from_utf8(&body[..]) {
//             Ok(s) => {
//                 assert!(s.contains("Jack"));
//                 assert!(s.contains("25"));
//                 assert!(s.contains("true"));
//             }, // Output: String: Hello
//             Err(e) => panic!("Error: {}", e),
//         }
//     }
//}