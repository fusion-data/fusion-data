//! tests/api_credentials_test.rs
//! `cargo test -p hetumind --test api_credentials_test -- --nocapture`

use fusion_common::model::IdUuidResult;
use hetumind_core::{credential::CredentialId, workflow::CredentialKind};
use hetumind_studio::domain::credential::CredentialVerifyResult;
use http::StatusCode;
use serde_json::json;
use uuid::Uuid;

mod common;

use common::get_server;

/// Creates test credential data for testing
fn create_test_credential_json() -> (CredentialId, serde_json::Value) {
  let id = CredentialId::now_v7();
  let credential_json = json!({
    "id": id,
    "namespace_id": "test-namespace",
    "name": "Test API Key",
    "data": "{\"api_key\":\"sk-test123456789\",\"endpoint\":\"https://api.example.com\"}",
    "kind": CredentialKind::GenericAuth,
    "is_managed": false
  });
  (id, credential_json)
}

/// Creates test OAuth2 credential data
fn create_test_oauth2_credential_json() -> (Uuid, serde_json::Value) {
  let id = Uuid::now_v7();
  let credential_json = json!({
    "id": id,
    "namespace_id": "test-namespace",
    "name": "Test OAuth2 App",
    "data": "{\"client_id\":\"test-client\",\"client_secret\":\"test-secret\",\"redirect_uri\":\"https://example.com/callback\"}",
    "kind": CredentialKind::Oauth2,
    "is_managed": true
  });
  (id, credential_json)
}

/// Creates test credential verification request
fn create_test_verify_request(kind: CredentialKind) -> serde_json::Value {
  json!({
    "data": {
      "data": "{\"api_key\":\"sk-test123456789\",\"endpoint\":\"https://api.example.com\"}",
      "test_connection": true
    },
    "kind": kind
  })
}

/// cargo test -p hetumind --test api_credentials_test -- --nocapture --ignored test_create_get_delete_credential
#[tokio::test]
#[ignore]
async fn test_create_get_delete_credential() {
  let server = get_server().await;

  let (credential_id, credential_json) = create_test_credential_json();

  println!("JONS: {}", credential_json);
  // 1. Create a new credential
  let response = server.post("/v1/credentials").json(&credential_json).await;
  response.assert_status_ok();
  let res = response.json::<IdUuidResult>();
  assert_eq!(res.id.to_string(), credential_id.to_string());

  // 2. Get the credential by ID
  let response = server.get(&format!("/v1/credentials/{}", credential_id)).await;
  response.assert_status_ok();
  let credential: serde_json::Value = response.json();
  assert_eq!(credential["credential"]["id"], credential_id.to_string());
  assert_eq!(credential["credential"]["name"], "Test API Key");
  assert_eq!(credential["credential"]["kind"], serde_json::to_value(CredentialKind::GenericAuth).unwrap());
  assert_eq!(credential["credential"]["is_managed"], false);

  // 3. Delete the credential
  let response = server.delete(&format!("/v1/credentials/{}", credential_id)).await;
  response.assert_status_ok();

  // 4. Verify the credential is deleted
  let response = server.get(&format!("/v1/credentials/{}", credential_id)).await;
  response.assert_status_not_found();
}

/// cargo test -p hetumind --test api_credentials_test -- --nocapture --ignored test_update_credential
#[tokio::test]
#[ignore]
async fn test_update_credential() {
  let server = get_server().await;

  let (credential_id, credential_json) = create_test_credential_json();

  // 1. Create a credential
  let response = server.post("/v1/credentials").json(&credential_json).await;
  let res = response.json::<IdUuidResult>();
  assert_eq!(res.id.to_string(), credential_id.to_string());

  // 2. Update the credential
  let update_data = json!({
    "name": "Updated API Key",
    "data": "{\"api_key\":\"sk-updated123\",\"endpoint\":\"https://api.updated.com\"}",
    "is_managed": true
  });
  let response = server.put(&format!("/v1/credentials/{}", credential_id)).json(&update_data).await;
  response.assert_status_ok();

  // 3. Get the credential to verify changes
  let response = server.get(&format!("/v1/credentials/{}", credential_id)).await;
  let credential: serde_json::Value = response.json();
  assert_eq!(credential["credential"]["name"], "Updated API Key");
  assert_eq!(credential["credential"]["is_managed"], true);

  // 4. Delete the credential
  let response = server.delete(&format!("/v1/credentials/{}", credential_id)).await;
  response.assert_status_ok();
}

/// cargo test -p hetumind --test api_credentials_test -- --nocapture --ignored test_query_credentials
#[tokio::test]
#[ignore]
async fn test_query_credentials() {
  let server = get_server().await;

  // Create multiple credentials for testing
  let (cred1_id, cred1_json) = create_test_credential_json();
  let (cred2_id, cred2_json) = create_test_oauth2_credential_json();

  // Create credentials
  let response = server.post("/v1/credentials").json(&cred1_json).await;
  response.assert_status_ok();
  let response = server.post("/v1/credentials").json(&cred2_json).await;
  response.assert_status_ok();

  // Query all credentials
  let query_data = json!({
    "page": {
      "page": 1,
      "limit": 10
    },
    "filters": []
  });
  let response = server.post("/v1/credentials/query").json(&query_data).await;
  response.assert_status_ok();
  let result: serde_json::Value = response.json();
  assert!(result["page"]["total"].as_u64().unwrap_or(0) >= 2);

  // Query credentials by name filter
  let query_data = json!({
    "page": {
      "page": 1,
      "limit": 10
    },
    "filters": [{
      "name": {
        "$like": "%API Key%"
      }
    }]
  });
  let response = server.post("/v1/credentials/query").json(&query_data).await;
  response.assert_status_ok();
  let result: serde_json::Value = response.json();
  assert!(result["page"]["total"].as_u64().unwrap_or(0) >= 1);

  // Query credentials by kind filter
  let query_data = json!({
    "page": {
      "page": 1,
      "limit": 10
    },
    "filters": [{
      "kind": CredentialKind::Oauth2
    }]
  });
  let response = server.post("/v1/credentials/query").json(&query_data).await;
  response.assert_status_ok();
  let result: serde_json::Value = response.json();
  assert!(result["page"]["total"].as_u64().unwrap_or(0) >= 1);

  // Cleanup
  let response = server.delete(&format!("/v1/credentials/{}", cred1_id)).await;
  response.assert_status_ok();
  let response = server.delete(&format!("/v1/credentials/{}", cred2_id)).await;
  response.assert_status_ok();
}

/// cargo test -p hetumind --test api_credentials_test -- --nocapture --ignored test_verify_credential
#[tokio::test]
#[ignore]
async fn test_verify_credential() {
  let server = get_server().await;

  // Test verification of unsaved credential
  let verify_request = create_test_verify_request(CredentialKind::GenericAuth);
  let response = server.post("/v1/credentials/verify").json(&verify_request).await;
  response.assert_status_ok();
  let result: CredentialVerifyResult = response.json();
  // The verification result depends on the actual implementation
  // We're testing that the endpoint responds correctly
  assert!(result.verify_time.timestamp() > 0);
}

/// cargo test -p hetumind --test api_credentials_test -- --nocapture --ignored test_verify_stored_credential
#[tokio::test]
#[ignore]
async fn test_verify_stored_credential() {
  let server = get_server().await;

  let (credential_id, credential_json) = create_test_credential_json();

  // 1. Create a credential
  let response = server.post("/v1/credentials").json(&credential_json).await;
  response.assert_status_ok();

  // 2. Verify the stored credential
  let response = server.post(&format!("/v1/credentials/{}/verify", credential_id)).await;
  response.assert_status_ok();
  let result: CredentialVerifyResult = response.json();
  assert!(result.verify_time.timestamp() > 0);

  // 3. Cleanup
  let response = server.delete(&format!("/v1/credentials/{}", credential_id)).await;
  response.assert_status_ok();
}

/// cargo test -p hetumind --test api_credentials_test -- --nocapture --ignored test_credential_not_found
#[tokio::test]
#[ignore]
async fn test_credential_not_found() {
  let server = get_server().await;

  let non_existent_id = Uuid::now_v7();

  // Test getting non-existent credential
  let response = server.get(&format!("/v1/credentials/{}", non_existent_id)).await;
  response.assert_status_not_found();

  // Test updating non-existent credential
  let update_data = json!({});
  let response = server.put(&format!("/v1/credentials/{}", non_existent_id)).json(&update_data).await;
  response.assert_status_not_found();

  // Test deleting non-existent credential
  let response = server.delete(&format!("/v1/credentials/{}", non_existent_id)).await;
  response.assert_status_not_found();

  // Test verifying non-existent credential
  let response = server.post(&format!("/v1/credentials/{}/verify", non_existent_id)).await;
  response.assert_status_not_found();
}

/// cargo test -p hetumind --test api_credentials_test -- --nocapture --ignored test_invalid_credential_data
#[tokio::test]
#[ignore]
async fn test_invalid_credential_data() {
  let server = get_server().await;

  // Test creating credential with missing required fields
  let invalid_credential = json!({
    "name": "Invalid Credential"
    // Missing namespace_id, data, and kind
  });
  let response = server.post("/v1/credentials").json(&invalid_credential).await;
  response.assert_status(StatusCode::BAD_REQUEST);

  // Test creating credential with invalid kind
  let invalid_credential = json!({
    "namespace_id": "test-namespace",
    "name": "Invalid Credential",
    "data": "{\"test\":\"data\"}",
    "kind": -1
  });
  let response = server.post("/v1/credentials").json(&invalid_credential).await;
  response.assert_status(StatusCode::BAD_REQUEST);

  // Test verification with invalid data
  let invalid_verify_request = json!({
    "data": {
      "data": "invalid json",
      "test_connection": true
    },
    "kind": CredentialKind::GenericAuth
  });
  let response = server.post("/v1/credentials/verify").json(&invalid_verify_request).await;
  response.assert_status(StatusCode::BAD_REQUEST);
}

/// cargo test -p hetumind --test api_credentials_test -- --nocapture --ignored test_oauth2_credential_lifecycle
#[tokio::test]
#[ignore]
async fn test_oauth2_credential_lifecycle() {
  let server = get_server().await;

  let (credential_id, credential_json) = create_test_oauth2_credential_json();

  // 1. Create OAuth2 credential
  let response = server.post("/v1/credentials").json(&credential_json).await;
  response.assert_status_ok();
  let res = response.json::<IdUuidResult>();
  assert_eq!(res.id.to_string(), credential_id.to_string());

  // 2. Get OAuth2 credential
  let response = server.get(&format!("/v1/credentials/{}", credential_id)).await;
  response.assert_status_ok();
  let credential: serde_json::Value = response.json();
  assert_eq!(credential["credential"]["kind"], "oauth2");
  assert_eq!(credential["credential"]["is_managed"], true);

  // 3. Update OAuth2 credential
  let update_data = json!({
    "name": "Updated OAuth2 App",
    "data": "{\"client_id\":\"updated-client\",\"client_secret\":\"updated-secret\"}"
  });
  let response = server.put(&format!("/v1/credentials/{}", credential_id)).json(&update_data).await;
  response.assert_status_ok();

  // 4. Verify updated credential
  let response = server.get(&format!("/v1/credentials/{}", credential_id)).await;
  let credential: serde_json::Value = response.json();
  assert_eq!(credential["credential"]["name"], "Updated OAuth2 App");

  // 5. Delete OAuth2 credential
  let response = server.delete(&format!("/v1/credentials/{}", credential_id)).await;
  response.assert_status_ok();
}

/// cargo test -p hetumind --test api_credentials_test -- --nocapture --ignored test_credential_pagination
#[tokio::test]
#[ignore]
async fn test_credential_pagination() {
  let server = get_server().await;

  let mut credential_ids = Vec::new();

  // Create multiple credentials for pagination testing
  for i in 0..5 {
    let (id, mut credential_json) = create_test_credential_json();
    credential_json["name"] = serde_json::Value::String(format!("Test Credential {}", i));

    let response = server.post("/v1/credentials").json(&credential_json).await;
    response.assert_status_ok();
    credential_ids.push(id);
  }

  // Test first page
  let query_data = json!({
    "page": {
      "page": 1,
      "limit": 2
    },
    "filters": []
  });
  let response = server.post("/v1/credentials/query").json(&query_data).await;
  response.assert_status_ok();
  let result: serde_json::Value = response.json();
  assert_eq!(result["result"].as_array().unwrap().len(), 2);
  assert!(result["page"]["total"].as_u64().unwrap_or(0) >= 5);

  // Test second page
  let query_data = json!({
    "page": {
      "page": 2,
      "limit": 2
    },
    "filters": []
  });
  let response = server.post("/v1/credentials/query").json(&query_data).await;
  response.assert_status_ok();
  let result: serde_json::Value = response.json();
  assert_eq!(result["result"].as_array().unwrap().len(), 2);

  // Cleanup
  for id in credential_ids {
    let response = server.delete(&format!("/v1/credentials/{}", id)).await;
    response.assert_status_ok();
  }
}
