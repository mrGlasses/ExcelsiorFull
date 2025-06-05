use crate::utils::un_utils::*;

#[tokio::test]
async fn test_start_message() {
    start_message("8080".to_string()).await;
    assert!(true);
}
