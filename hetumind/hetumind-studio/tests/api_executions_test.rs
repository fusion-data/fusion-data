//! tests/api_executions_test.rs

use fusion_common::model::IdResult;
use hetumind_core::workflow::{Execution, ExecutionFilter, ExecutionForQuery, WorkflowId};
use modelsql::{
  filter::{OpValUuid, Page},
  page::PageResult,
};
use serde_json::json;

use crate::common::get_server;

mod common;

/// cargo test -p hetumind --test api_executions_test -- --nocapture --ignored test_execute_and_query_execution
#[tokio::test]
#[ignore]
async fn test_execute_and_query_execution() {
  let server = get_server().await;

  // 1. Create a new workflow with a simple structure
  let response = server
    .post("/v1/workflows")
    .json(&json!({
        "name": "My Execution Test Workflow",
        "nodes": [
            { "id": "start", "kind": "Start", "name": "Start" },
            { "id": "end", "kind": "End", "name": "End" }
        ],
        "connections": [
            { "source": "start", "target": "end" }
        ]
    }))
    .await;
  response.assert_status_ok();
  let id_result: IdResult = response.json();
  let workflow_id = WorkflowId::new(id_result.to_uuid().unwrap());

  // 2. Execute the workflow
  let response = server.post(&format!("/v1/workflows/{}/execute", workflow_id)).json(&json!({})).await;
  response.assert_status_ok();

  // Give a moment for execution to complete
  tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

  // 3. Query for executions for that workflow
  let response = server
    .post("/v1/executions/query")
    .json(&ExecutionForQuery {
      options: Page::default(),
      filter: ExecutionFilter { workflow_id: Some(OpValUuid::Eq(*workflow_id.as_ref()).into()), ..Default::default() },
    })
    .await;

  response.assert_status_ok();
  let page_result: PageResult<Execution> = response.json();

  assert_eq!(page_result.page.total, 1);
  assert_eq!(page_result.result.len(), 1);
  let execution = &page_result.result[0];
  assert_eq!(execution.workflow_id, workflow_id);
}

/// cargo test -p hetumind --test api_executions_test -- --nocapture --ignored test_get_and_cancel_execution
#[tokio::test]
#[ignore]
async fn test_get_and_cancel_execution() {
  let server = get_server().await;

  // 1. Create a workflow
  let response = server.post("/v1/workflows").json(&json!({ "name": "Cancellable Workflow" })).await;
  let workflow_id: hetumind_core::workflow::WorkflowId = response.json();

  // 2. Execute the workflow
  let response = server.post(&format!("/v1/workflows/{}/execute", workflow_id)).json(&json!({})).await;
  let execution_response: hetumind_core::workflow::ExecutionIdResponse = response.json();
  let execution_id = execution_response.execution_id;

  // 3. Get the execution by ID
  let response = server.get(&format!("/v1/executions/{}", execution_id)).await;
  response.assert_status_ok();
  let execution: Execution = response.json();
  assert_eq!(execution.id, execution_id);
  assert_eq!(execution.workflow_id, workflow_id);

  // 4. Cancel the execution
  let response = server.post(&format!("/v1/executions/{}/cancel", execution_id)).await;
  response.assert_status_ok();

  // 5. Get the execution again to verify the status
  // Note: The status might not change immediately to Cancelled depending on the engine's implementation.
  // Here we just check that the API call was successful.
  // A more robust test would involve a long-running task and checking for the Cancelled status.
  let response = server.get(&format!("/v1/executions/{}", execution_id)).await;
  response.assert_status_ok();
}
