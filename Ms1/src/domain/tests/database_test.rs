use crate::domain::database::{NewUser, User};
use serde_json::json;

#[test]
fn test_new_user_deserialization() {
    let json_data = json!({
        "name": "John Doe"
    });
    
    let new_user: NewUser = serde_json::from_value(json_data).unwrap();
    assert_eq!(new_user.name, "John Doe");
}

#[test]
fn test_new_user_missing_field() {
    let json_data = json!({});
    let result = serde_json::from_value::<NewUser>(json_data);
    assert!(result.is_err());
}

#[test]
fn test_user_serialization() {
    let user = User {
        uid: 1,
        name: String::from("John Doe"),
    };
    
    let serialized = serde_json::to_value(&user).unwrap();
    let expected = json!({
        "uid": 1,
        "name": "John Doe"
    });
    
    assert_eq!(serialized, expected);
}

#[test]
fn test_user_deserialization() {
    let json_data = json!({
        "uid": 1,
        "name": "John Doe"
    });
    
    let user: User = serde_json::from_value(json_data).unwrap();
    assert_eq!(user.uid, 1);
    assert_eq!(user.name, "John Doe");
}

#[test]
fn test_user_clone() {
    let original_user = User {
        uid: 1,
        name: String::from("John Doe"),
    };
    
    let cloned_user = original_user.clone();
    
    assert_eq!(original_user.uid, cloned_user.uid);
    assert_eq!(original_user.name, cloned_user.name);
}

#[test]
fn test_user_missing_fields() {
    let json_data = json!({
        "uid": 1
    });
    
    let result = serde_json::from_value::<User>(json_data);
    assert!(result.is_err());
}

#[test]
fn test_user_invalid_types() {
    let json_data = json!({
        "uid": "not_a_number",
        "name": "John Doe"
    });
    
    let result = serde_json::from_value::<User>(json_data);
    assert!(result.is_err());
} 