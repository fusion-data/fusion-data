//! Test utilities and helpers
//!
//! This module provides common utilities and mock implementations
//! for testing the API implementations.

use fusion_common::time::now_utc;
use hetumind_core::user::{UserEntity, UserStatus};
use serde_json::Value;

/// Mock application for testing
pub struct MockApplication {
    pub config: MockConfig,
}

impl MockApplication {
    pub fn new() -> Self {
        Self {
            config: MockConfig::default(),
        }
    }
}

/// Mock configuration
#[derive(Default)]
pub struct MockConfig {
    pub security: MockSecurityConfig,
}

/// Mock security configuration
#[derive(Default)]
pub struct MockSecurityConfig {
    pub pwd_expires_at: fusion_common::time::OffsetDateTime,
}

impl MockSecurityConfig {
    pub fn pwd(&self) -> MockPwdConfig {
        MockPwdConfig {
            expires_at: self.pwd_expires_at,
        }
    }
}

/// Mock password configuration
pub struct MockPwdConfig {
    pub expires_at: fusion_common::time::OffsetDateTime,
}

impl MockPwdConfig {
    pub fn expires_at(&self) -> fusion_common::time::OffsetDateTime {
        self.expires_at
    }
}

/// Create a test user entity
pub fn create_test_user(id: i64, email: &str, password: Option<String>) -> UserEntity {
    UserEntity {
        id,
        email: email.to_string(),
        phone: None,
        name: Some(format!("Test User {}", id)),
        password,
        status: UserStatus::Enabled,
        created_at: now_utc(),
        updated_at: now_utc(),
    }
}

/// Create a test JSON object
pub fn create_test_json() -> Value {
    serde_json::json!({
        "test": "value",
        "number": 42,
        "array": [1, 2, 3],
        "nested": {
            "key": "value"
        }
    })
}

/// Test helper to validate UUID format
pub fn is_valid_uuid(uuid_str: &str) -> bool {
    uuid::Uuid::parse_str(uuid_str).is_ok()
}

/// Test helper to validate timestamp format
pub fn is_valid_timestamp(timestamp: &str) -> bool {
    // Try to parse as ISO 8601 timestamp
    chrono::DateTime::parse_from_rfc3339(timestamp).is_ok()
}

/// Test helper to generate random email
pub fn generate_random_email() -> String {
    format!("test{}@example.com", uuid::Uuid::new_v4())
}

/// Test helper to generate random password
pub fn generate_random_password() -> String {
    use rand::Rng;
    let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let mut rng = rand::thread_rng();

    (0..16)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_user() {
        let user = create_test_user(1, "test@example.com", Some("password".to_string()));

        assert_eq!(user.id, 1);
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, Some("Test User 1".to_string()));
        assert_eq!(user.status, UserStatus::Enabled);
        assert!(user.password.is_some());
    }

    #[test]
    fn test_create_test_json() {
        let json = create_test_json();

        assert_eq!(json["test"], "value");
        assert_eq!(json["number"], 42);
        assert_eq!(json["array"][0], 1);
        assert_eq!(json["nested"]["key"], "value");
    }

    #[test]
    fn test_is_valid_uuid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let invalid_uuid = "not-a-uuid";

        assert!(is_valid_uuid(valid_uuid));
        assert!(!is_valid_uuid(invalid_uuid));
    }

    #[test]
    fn test_is_valid_timestamp() {
        let valid_timestamp = "2024-01-01T00:00:00Z";
        let invalid_timestamp = "not-a-timestamp";

        assert!(is_valid_timestamp(valid_timestamp));
        assert!(!is_valid_timestamp(invalid_timestamp));
    }

    #[test]
    fn test_generate_random_email() {
        let email = generate_random_email();
        assert!(email.contains("@example.com"));
        assert!(email.len() > 20); // UUID length + @example.com
    }

    #[test]
    fn test_generate_random_password() {
        let password = generate_random_password();
        assert_eq!(password.len(), 16);

        // Check if it contains different character types
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| "!@#$%^&*".contains(c));

        assert!(has_upper || has_lower); // At least some letters
        assert!(has_digit); // At least one digit
    }
}