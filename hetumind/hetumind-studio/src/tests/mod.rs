//! Test module for hetumind-studio
//!
//! This module contains unit tests for the various API implementations.
//! Tests are organized by feature area.

// Test data models and utilities
pub mod test_utils;

// Import individual test modules
pub use auth_tests::*;
pub use executions_tests::*;
pub use credentials_tests::*;
pub use users_tests::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test_module_compilation() {
        // This is a simple test to ensure the test module compiles correctly
        assert!(true);
    }
}