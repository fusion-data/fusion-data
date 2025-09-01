//! tests/api_workflows_test.rs
//! `cargo test -p hetumind --test api_workflows_test -- --nocapture`

use hetumind_core::workflow::{Workflow, WorkflowForUpdate, WorkflowStatus};
use http::StatusCode;
use serde_json::json;
use fusion_core::IdUuidResult;

mod common;

use common::{create_test_workflow_json, get_server};

/// cargo test -p hetumind --test api_workflows_test -- --nocapture --ignored test_create_get_delete_workflow
#[tokio::test]
#[ignore]
async fn test_create_get_delete_workflow() {
  let server = get_server().await;

  let (workflow_id, workflow_json) = create_test_workflow_json();

  // 1. Create a new workflow
  let response = server.post("/v1/workflows").json(&workflow_json).await;

  response.assert_status_ok();
  let res = response.json::<IdUuidResult>();
  assert_eq!(&res.id, workflow_id.as_ref());

  // 2. Get the workflow by ID
  let response = server.get(&format!("/v1/workflows/{}", workflow_id)).await;
  response.assert_status_ok();
  let workflow: Workflow = response.json();
  assert_eq!(workflow.id, workflow_id);
  assert_eq!(workflow.name, "My Test Workflow");

  // 3. Delete the workflow
  let response = server.delete(&format!("/v1/workflows/{}", workflow_id)).await;
  response.assert_status_ok();

  // 4. Verify the workflow is deleted
  let response = server.get(&format!("/v1/workflows/{}", workflow_id)).await;
  response.assert_status_not_found();
}

/// cargo test -p hetumind --test api_workflows_test -- --nocapture --ignored test_update_workflow
#[tokio::test]
#[ignore]
async fn test_update_workflow() {
  let server = get_server().await;

  let (workflow_id, workflow_json) = create_test_workflow_json();

  // 1. Create a workflow
  let response = server.post("/v1/workflows").json(&workflow_json).await;
  let res = response.json::<IdUuidResult>();
  assert_eq!(&res.id, workflow_id.as_ref());

  // 2. Update the workflow
  let update_data = WorkflowForUpdate {
    name: Some("Updated Workflow Name".to_string()),
    status: Some(WorkflowStatus::Disabled),
    ..Default::default()
  };
  let response = server.put(&format!("/v1/workflows/{}", workflow_id)).json(&update_data).await;
  response.assert_status_ok();

  // 3. Get the workflow to verify changes
  let response = server.get(&format!("/v1/workflows/{}", workflow_id)).await;
  let workflow: Workflow = response.json();
  assert_eq!(workflow.name, "Updated Workflow Name");
  assert_eq!(workflow.status, WorkflowStatus::Disabled);

  // 4. Delete the workflow
  let response = server.delete(&format!("/v1/workflows/{}", workflow_id)).await;
  response.assert_status_ok();
}

/// cargo test -p hetumind --test api_workflows_test -- --nocapture --ignored test_activate_deactivate_workflow
#[tokio::test]
#[ignore]
async fn test_activate_deactivate_workflow() {
  let server = get_server().await;

  let (workflow_id, workflow_json) = create_test_workflow_json();
  // 1. Create a workflow, it should be in Draft status by default
  let response = server.post("/v1/workflows").json(&workflow_json).await;
  let res = response.json::<IdUuidResult>();
  assert_eq!(&res.id, workflow_id.as_ref());

  let response = server.get(&format!("/v1/workflows/{}", workflow_id)).await;
  let workflow: Workflow = response.json();
  assert_eq!(workflow.status, WorkflowStatus::Draft);

  // 2. Activate the workflow
  server.post(&format!("/v1/workflows/{}/activate", workflow_id)).await.assert_status_ok();

  let response = server.get(&format!("/v1/workflows/{}", workflow_id)).await;
  let workflow: Workflow = response.json();
  assert_eq!(workflow.status, WorkflowStatus::Active);

  // 3. Deactivate the workflow
  server.post(&format!("/v1/workflows/{}/deactivate", workflow_id)).await.assert_status_ok();

  let response = server.get(&format!("/v1/workflows/{}", workflow_id)).await;
  let workflow: Workflow = response.json();
  assert_eq!(workflow.status, WorkflowStatus::Disabled);
}

/// cargo test -p hetumind --test api_workflows_test -- --nocapture --ignored test_duplicate_workflow
#[tokio::test]
#[ignore]
async fn test_duplicate_workflow() {
  let server = get_server().await;

  let (original_id, workflow_json) = create_test_workflow_json();

  // 1. Create a workflow
  let response = server.post("/v1/workflows").json(&workflow_json).await;
  let res = response.json::<IdUuidResult>();
  assert_eq!(&res.id, original_id.as_ref());

  // 2. Duplicate it
  let response = server.post(&format!("/v1/workflows/{}/duplicate", original_id)).await;
  response.assert_status_ok();
  let IdUuidResult { id: new_id } = response.json();
  assert_ne!(original_id.as_ref(), &new_id);

  // 3. Get both and compare
  let response = server.get(&format!("/v1/workflows/{}", original_id)).await;
  let original_workflow: Workflow = response.json();

  let response = server.get(&format!("/v1/workflows/{}", new_id)).await;
  let new_workflow: Workflow = response.json();

  assert_eq!(new_workflow.name, format!("{} (Copy)", original_workflow.name));
  assert_eq!(new_workflow.nodes.len(), original_workflow.nodes.len());
  assert_eq!(new_workflow.status, WorkflowStatus::Draft); // a new duplicated workflow should be a draft
}

/// cargo test -p hetumind --test api_workflows_test -- --nocapture --ignored test_validate_workflow
#[tokio::test]
#[ignore]
async fn test_validate_workflow() {
  let server = get_server().await;

  let (_original_id, mut workflow_json) = create_test_workflow_json();

  // A valid workflow
  json_patch::merge(
    &mut workflow_json,
    &json!({
      "name": "Valid Workflow",
      "nodes": [
        { "id": "start", "kind": "Start", "name": "Start" },
        { "id": "end", "kind": "End", "name": "End" }
      ],
      "connections": [
        { "source": "start", "target": "end" }
      ]
    }),
  );

  let response = server.post("/v1/workflows/validate").json(&workflow_json).await;
  response.assert_status_ok();

  // An invalid workflow (unknown node kind)
  let response = server
    .post("/v1/workflows/validate")
    .json(&json!({
        "name": "Invalid Workflow",
        "nodes": [
            { "id": "start", "kind": "UnknownKind", "name": "Start" },
        ],
    }))
    .await;
  response.assert_status(StatusCode::BAD_REQUEST); // Bad Request
}
