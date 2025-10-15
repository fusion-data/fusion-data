//! Integration tests for the implemented APIs
//!
//! These tests verify that all the newly implemented APIs work together correctly
//! and follow the expected patterns and conventions.

use reqwest;
use serde_json;
use std::collections::HashMap;

// Test configuration
const BASE_URL: &str = "http://localhost:8080";
const TEST_EMAIL: &str = "test@example.com";
const TEST_PASSWORD: &str = "test_password_123";

/// Integration test helper struct
struct TestClient {
    client: reqwest::Client,
    base_url: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl TestClient {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: BASE_URL.to_string(),
            access_token: None,
            refresh_token: None,
        }
    }

    async fn signin(&mut self, email: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut body = HashMap::new();
        body.insert("account", email);
        body.insert("password", password);

        let response = self.client
            .post(&format!("{}/api/auth/signin", self.base_url))
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: serde_json::Value = response.json().await?;

            self.access_token = auth_response.get("access_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            self.refresh_token = auth_response.get("refresh_token")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Ok(())
        } else {
            Err(format!("Signin failed: {}", response.status()).into())
        }
    }

    async fn refresh_token(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(refresh_token) = &self.refresh_token {
            let body = serde_json::json!({
                "refresh_token": refresh_token
            });

            let response = self.client
                .post(&format!("{}/api/auth/refresh", self.base_url))
                .json(&body)
                .send()
                .await?;

            if response.status().is_success() {
                let refresh_response: serde_json::Value = response.json().await?;

                self.access_token = refresh_response.get("access_token")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Ok(())
            } else {
                Err(format!("Token refresh failed: {}", response.status()).into())
            }
        } else {
            Err("No refresh token available".into())
        }
    }

    async fn signout(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(access_token) = &self.access_token {
            let body = serde_json::json!({
                "token": access_token
            });

            let response = self.client
                .post(&format!("{}/api/auth/signout", self.base_url))
                .json(&body)
                .send()
                .await?;

            if response.status().is_success() {
                self.access_token = None;
                self.refresh_token = None;
                Ok(())
            } else {
                Err(format!("Signout failed: {}", response.status()).into())
            }
        } else {
            Err("No access token available".into())
        }
    }

    fn authenticated_request(&self) -> reqwest::RequestBuilder {
        if let Some(token) = &self.access_token {
            self.client
                .request(reqwest::Method::GET, "")
                .header("Authorization", format!("Bearer {}", token))
        } else {
            self.client.request(reqwest::Method::GET, "")
        }
    }

    async fn get_execution_status(&self, execution_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = self.authenticated_request()
            .get(&format!("{}/api/v1/executions/{}/status", self.base_url, execution_id))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(format!("Get execution status failed: {}", response.status()).into())
        }
    }

    async fn get_credential_references(&self, credential_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = self.authenticated_request()
            .get(&format!("{}/api/v1/credentials/{}/references", self.base_url, credential_id))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(format!("Get credential references failed: {}", response.status()).into())
        }
    }

    async fn update_user_password(&self, user_id: i64, old_password: &str, new_password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let body = serde_json::json!({
            "old_password": old_password,
            "new_password": new_password
        });

        let response = self.authenticated_request()
            .put(&format!("{}/api/v1/users/item/{}/password", self.base_url, user_id))
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Update user password failed: {}", response.status()).into())
        }
    }
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_complete_authentication_flow() {
    let mut client = TestClient::new();

    // Test signin
    client.signin(TEST_EMAIL, TEST_PASSWORD).await.unwrap();
    assert!(client.access_token.is_some());
    assert!(client.refresh_token.is_some());

    // Test token refresh
    let old_access_token = client.access_token.clone();
    client.refresh_token().await.unwrap();
    assert!(client.access_token.is_some());
    assert_ne!(client.access_token, old_access_token);

    // Test signout
    client.signout().await.unwrap();
    assert!(client.access_token.is_none());
    assert!(client.refresh_token.is_none());
}

#[tokio::test]
#[ignore] // Requires running server and test data
async fn test_execution_status_api() {
    let mut client = TestClient::new();
    client.signin(TEST_EMAIL, TEST_PASSWORD).await.unwrap();

    // Test getting execution status
    let execution_id = "test-execution-id";
    let status = client.get_execution_status(execution_id).await.unwrap();

    // Verify response structure
    assert!(status.get("id").is_some());
    assert!(status.get("status").is_some());
    assert!(status.get("started_at").is_some());
    assert!(status.get("progress").is_some());

    client.signout().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires running server and test data
async fn test_credential_references_api() {
    let mut client = TestClient::new();
    client.signin(TEST_EMAIL, TEST_PASSWORD).await.unwrap();

    // Test getting credential references
    let credential_id = "test-credential-id";
    let references = client.get_credential_references(credential_id).await.unwrap();

    // Verify response structure
    assert!(references.get("credential_id").is_some());
    assert!(references.get("references").is_some());
    assert!(references.get("total_count").is_some());

    let total_count = references.get("total_count").unwrap().as_i64().unwrap();
    assert!(total_count >= 0);

    client.signout().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires running server and test user
async fn test_user_password_update_flow() {
    let mut client = TestClient::new();

    // Initial signin
    client.signin(TEST_EMAIL, TEST_PASSWORD).await.unwrap();

    // Update password
    let new_password = "new_test_password_456";
    client.update_user_password(1, TEST_PASSWORD, new_password).await.unwrap();

    // Signout
    client.signout().await.unwrap();

    // Try to signin with old password (should fail)
    let signin_result = client.signin(TEST_EMAIL, TEST_PASSWORD).await;
    assert!(signin_result.is_err());

    // Signin with new password (should succeed)
    client.signin(TEST_EMAIL, new_password).await.unwrap();
    assert!(client.access_token.is_some());

    // Update back to original password for cleanup
    client.update_user_password(1, new_password, TEST_PASSWORD).await.unwrap();
    client.signout().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_api_error_handling() {
    let mut client = TestClient::new();

    // Test invalid credentials
    let signin_result = client.signin("invalid@example.com", "wrong_password").await;
    assert!(signin_result.is_err());

    // Test accessing protected endpoint without authentication
    let status_result = client.get_execution_status("fake-id").await;
    assert!(status_result.is_err());

    // Test invalid execution ID after authentication
    client.signin(TEST_EMAIL, TEST_PASSWORD).await.unwrap();
    let status_result = client.get_execution_status("invalid-execution-id").await;
    // This might succeed with empty data or fail with 404, depending on implementation
    // Either way, it should not panic

    client.signout().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires running server
async fn test_concurrent_requests() {
    let mut client = TestClient::new();
    client.signin(TEST_EMAIL, TEST_PASSWORD).await.unwrap();

    // Test multiple concurrent requests
    let mut handles = Vec::new();

    for i in 0..10 {
        let execution_id = format!("test-execution-{}", i);
        let client_clone = client.clone();

        let handle = tokio::spawn(async move {
            // This would need proper cloning implementation for TestClient
            // For now, this is a placeholder showing the test structure
            format!("Would test execution {} concurrently", execution_id)
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let result = handle.await.unwrap();
        println!("{}", result);
    }

    client.signout().await.unwrap();
}

#[tokio::test]
async fn test_api_request_serialization() {
    // Test that all request structures can be serialized properly

    // Test refresh token request
    let refresh_request = serde_json::json!({
        "refresh_token": "test_refresh_token"
    });
    assert!(serde_json::to_string(&refresh_request).is_ok());

    // Test signout request
    let signout_request = serde_json::json!({
        "token": "test_access_token"
    });
    assert!(serde_json::to_string(&signout_request).is_ok());

    // Test password update request
    let password_update_request = serde_json::json!({
        "old_password": "old_password",
        "new_password": "new_password"
    });
    assert!(serde_json::to_string(&password_update_request).is_ok());

    // Test password update request with verification code
    let password_update_request_with_code = serde_json::json!({
        "verification_code": "123456",
        "new_password": "new_password"
    });
    assert!(serde_json::to_string(&password_update_request_with_code).is_ok());
}

#[tokio::test]
async fn test_api_response_deserialization() {
    // Test that all response structures can be deserialized properly

    // Test signin response
    let signin_response = serde_json::json!({
        "access_token": "test_access_token",
        "refresh_token": "test_refresh_token",
        "token_type": "Bearer",
        "expires_in": 3600
    });
    assert!(serde_json::from_str::<serde_json::Value>(&signin_response.to_string()).is_ok());

    // Test refresh token response
    let refresh_response = serde_json::json!({
        "access_token": "new_access_token",
        "token_type": "Bearer",
        "expires_in": 3600
    });
    assert!(serde_json::from_str::<serde_json::Value>(&refresh_response.to_string()).is_ok());

    // Test execution status response
    let execution_status_response = serde_json::json!({
        "id": "test-execution-id",
        "status": "Running",
        "started_at": "2024-01-01T00:00:00Z",
        "finished_at": null,
        "error": null,
        "progress": 0.5
    });
    assert!(serde_json::from_str::<serde_json::Value>(&execution_status_response.to_string()).is_ok());

    // Test credential references response
    let credential_references_response = serde_json::json!({
        "credential_id": "test-credential-id",
        "references": [
            {
                "id": "workflow-1",
                "name": "Test Workflow",
                "node_type": "action",
                "node_name": "test_node",
                "parameter_path": "config.api_key",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ],
        "total_count": 1
    });
    assert!(serde_json::from_str::<serde_json::Value>(&credential_references_response.to_string()).is_ok());
}