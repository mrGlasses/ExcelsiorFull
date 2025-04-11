#[cfg(test)]
mod tests {
    use crate::models::user::User;

    #[test]
    fn test_user_struct_fields() {
        let user = User {
            id: 1,
            name: "Test User".into(),
            email: "test@example.com".into(),
        };
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
    }
}