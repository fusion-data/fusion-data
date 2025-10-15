//! # Integration Workflow Test
//!
//! Complete integration test using ManualTriggerNode, IfNode, EditFieldsNode, and ReadWriteFilesNode
//! with DefaultWorkflowEngine. This test demonstrates:
//! - Manual workflow triggering
//! - Conditional branching with IfNode
//! - Data transformation with EditFieldsNode
//! - File I/O operations with ReadWriteFilesNode
//! - Complete workflow execution with DefaultWorkflowEngine
//! Run this test
//! ```shell
//! cargo test -p hetumind-studio --test integration_workflow_test -- --nocapture
//! ```

mod common;

use common::TestContext;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use fusion_common::ahash::HashMap;
use fusion_common::ctx::{Ctx, CtxPayload};
use fusion_common::time::now;
use fusion_core::application::Application;
use hetumind_core::workflow::{
  Connection, ConnectionKind, Execution, ExecutionContext, ExecutionData, ExecutionDataItems, ExecutionDataMap,
  ExecutionId, ExecutionStatus, NodeExecutionStatus, NodeKind, NodeName, NodeRegistry, ParameterMap, PinData, Workflow,
  WorkflowExecutionError, WorkflowId, WorkflowMeta, WorkflowNode, WorkflowSettings, WorkflowStatus,
  WorkflowTriggerData,
};
use hetumind_nodes::constants::{
  EDIT_FIELDS_NODE_KIND, IF_NODE_KIND, MANUAL_TRIGGER_NODE_KIND, READ_WRITE_FILES_NODE_KIND,
};
use hetumind_nodes::core::{EditFieldsNode, IfNode, ReadWriteFilesNode};
use hetumind_nodes::trigger::ManualTriggerNode;
use hetumind_studio::runtime::workflow::WorkflowEngineService;
use hetumind_studio::runtime::{
  checkpoint::{CheckpointError, ExecutionCheckpoint},
  execution::ExecutionStore,
};
use mea::rwlock::RwLock;
use serde_json::json;
use uuid::Uuid;

// Mock ExecutionStore for testing
#[derive(Default)]
pub struct MockExecutionStore {
  executions: Arc<RwLock<HashMap<ExecutionId, Execution>>>,
  checkpoints: Arc<RwLock<HashMap<ExecutionId, ExecutionCheckpoint>>>,
}

#[async_trait]
impl ExecutionStore for MockExecutionStore {
  async fn save_execution(&self, execution: &Execution) -> Result<(), WorkflowExecutionError> {
    let mut executions = self.executions.write().await;
    executions.insert(execution.id.clone(), execution.clone());
    Ok(())
  }

  async fn get_execution(&self, id: &ExecutionId) -> Result<Option<Execution>, WorkflowExecutionError> {
    let executions = self.executions.read().await;
    Ok(executions.get(id).cloned())
  }

  async fn get_execution_status(&self, id: &ExecutionId) -> Result<ExecutionStatus, WorkflowExecutionError> {
    let executions = self.executions.read().await;
    Ok(executions.get(id).map(|e| e.status).unwrap_or(ExecutionStatus::Failed))
  }

  async fn update_execution_status(
    &self,
    id: &ExecutionId,
    status: ExecutionStatus,
  ) -> Result<(), WorkflowExecutionError> {
    let mut executions = self.executions.write().await;
    if let Some(execution) = executions.get_mut(id) {
      execution.status = status;
      if status == ExecutionStatus::Success || status == ExecutionStatus::Failed {
        execution.finished_at = Some(now());
      }
    }
    Ok(())
  }

  async fn save_checkpoint(&self, checkpoint: ExecutionCheckpoint) -> Result<(), CheckpointError> {
    let mut checkpoints = self.checkpoints.write().await;
    checkpoints.insert(checkpoint.execution_id.clone(), checkpoint);
    Ok(())
  }

  async fn load_latest_checkpoint(
    &self,
    execution_id: &ExecutionId,
  ) -> Result<Option<ExecutionCheckpoint>, CheckpointError> {
    let checkpoints = self.checkpoints.read().await;
    Ok(checkpoints.get(execution_id).map(|c| ExecutionCheckpoint {
      execution_id: c.execution_id.clone(),
      timestamp: c.timestamp,
      execution_state: c.execution_state.clone(),
      completed_nodes: c.completed_nodes.clone(),
      current_nodes: c.current_nodes.clone(),
      pending_tasks: c.pending_tasks.clone(),
      intermediate_data: c.intermediate_data.clone(),
    }))
  }
}

/// Create a comprehensive workflow that tests all required nodes
fn create_integration_workflow() -> Result<Workflow, Box<dyn std::error::Error>> {
  let workflow_id = WorkflowId::now_v7();

  // Create ManualTriggerNode
  let manual_trigger_node =
    WorkflowNode::new(NodeKind::from(MANUAL_TRIGGER_NODE_KIND), NodeName::from("manual_trigger"))
      .with_display_name("Manual Trigger")
      .with_parameters(create_manual_trigger_parameters());

  // Create IfNode for conditional branching
  let if_node = WorkflowNode::new(NodeKind::from(IF_NODE_KIND), NodeName::from("condition_check"))
    .with_display_name("Check Condition")
    .with_parameters(create_if_node_parameters());

  // Create EditFieldsNode for data transformation (true branch)
  let set_node_true = WorkflowNode::new(NodeKind::from(EDIT_FIELDS_NODE_KIND), NodeName::from("set_data_true"))
    .with_display_name("Set Data (True)")
    .with_parameters(create_set_node_parameters_true());

  // Create EditFieldsNode for data transformation (false branch)
  let set_node_false = WorkflowNode::new(NodeKind::from(EDIT_FIELDS_NODE_KIND), NodeName::from("set_data_false"))
    .with_display_name("Set Data (False)")
    .with_parameters(create_set_node_parameters_false());

  // Create ReadWriteFilesNode for file operations
  let file_node = WorkflowNode::new(NodeKind::from(READ_WRITE_FILES_NODE_KIND), NodeName::from("file_operations"))
    .with_display_name("File Operations")
    .with_parameters(create_file_node_parameters());

  // Create connections
  let mut connections = HashMap::default();

  // Manual trigger -> If node
  let trigger_connections = vec![Connection::new(NodeName::from("condition_check"), ConnectionKind::Main, 0)];
  let mut trigger_connection_map = HashMap::default();
  trigger_connection_map.insert(ConnectionKind::Main, trigger_connections);
  connections.insert(NodeName::from("manual_trigger"), trigger_connection_map);

  // If node -> Set nodes (true and false branches)
  let if_connections_true = vec![Connection::new(NodeName::from("set_data_true"), ConnectionKind::Main, 0)];
  let if_connections_false = vec![Connection::new(NodeName::from("set_data_false"), ConnectionKind::Main, 0)];
  let mut if_connection_map = HashMap::default();
  if_connection_map.insert(ConnectionKind::Main, if_connections_true);
  if_connection_map.insert(ConnectionKind::Main, if_connections_false);
  connections.insert(NodeName::from("condition_check"), if_connection_map);

  // Set nodes -> File node (both branches converge)
  let set_true_connections = vec![Connection::new(NodeName::from("file_operations"), ConnectionKind::Main, 0)];
  let mut set_true_connection_map = HashMap::default();
  set_true_connection_map.insert(ConnectionKind::Main, set_true_connections);
  connections.insert(NodeName::from("set_data_true"), set_true_connection_map);

  let set_false_connections = vec![Connection::new(NodeName::from("file_operations"), ConnectionKind::Main, 0)];
  let mut set_false_connection_map = HashMap::default();
  set_false_connection_map.insert(ConnectionKind::Main, set_false_connections);
  connections.insert(NodeName::from("set_data_false"), set_false_connection_map);

  let workflow = Workflow {
    id: workflow_id,
    name: "Integration Workflow Test".to_string(),
    status: WorkflowStatus::Active,
    version: None,
    settings: WorkflowSettings::default(),
    meta: WorkflowMeta::default(),
    nodes: vec![manual_trigger_node, if_node, set_node_true, set_node_false, file_node],
    connections,
    pin_data: PinData::default(),
    static_data: None,
  };

  Ok(workflow)
}

/// Create parameters for ManualTriggerNode
fn create_manual_trigger_parameters() -> ParameterMap {
  let mut params = serde_json::Map::new();
  params.insert("execution_mode".to_string(), json!("test"));
  params.insert("enabled".to_string(), json!(true));
  ParameterMap::new(params)
}

/// Create parameters for IfNode
fn create_if_node_parameters() -> ParameterMap {
  let mut params = serde_json::Map::new();

  // Create conditions that check if execution_mode is "test"
  // Using proper ConditionConfig format
  let conditions = json!([
    {
      "left": "{{ $json.execution_mode }}",
      "op": "eq",
      "right": "test",
      "data_type": {
        "type": "string"
      }
    }
  ]);

  params.insert("conditions".to_string(), conditions);
  params.insert("combination".to_string(), json!("and"));
  ParameterMap::new(params)
}

/// Create parameters for EditFieldsNode (true branch)
fn create_set_node_parameters_true() -> ParameterMap {
  let mut params = serde_json::Map::new();

  let operations = json!([
    {
      "field_path": "processed_at",
      "kind": "set",
      "value_source": "current_timestamp",
      "value": null
    },
    {
      "field_path": "branch",
      "kind": "set",
      "value_source": "static",
      "value": "true_branch"
    },
    {
      "field_path": "message",
      "kind": "set",
      "value_source": "static",
      "value": "Condition was true - processing data"
    }
  ]);

  params.insert("operations".to_string(), operations);
  ParameterMap::new(params)
}

/// Create parameters for EditFieldsNode (false branch)
fn create_set_node_parameters_false() -> ParameterMap {
  let mut params = serde_json::Map::new();

  let operations = json!([
    {
      "field_path": "processed_at",
      "kind": "set",
      "value_source": "current_timestamp",
      "value": null
    },
    {
      "field_path": "branch",
      "kind": "set",
      "value_source": "static",
      "value": "false_branch"
    },
    {
      "field_path": "message",
      "kind": "set",
      "value_source": "static",
      "value": "Condition was false - processing alternative data"
    }
  ]);

  params.insert("operations".to_string(), operations);
  ParameterMap::new(params)
}

/// Create parameters for ReadWriteFilesNode
fn create_file_node_parameters() -> ParameterMap {
  let mut params = serde_json::Map::new();

  // Write operation configuration matching the node definition
  params.insert("operation".to_string(), json!("write"));
  params.insert("file_path".to_string(), json!("test_output.json"));
  params.insert(
    "options".to_string(),
    json!({
      "append": false,
      "continue_on_fail": true
    }),
  );
  ParameterMap::new(params)
}

#[tokio::test]
async fn test_integration_workflow() -> Result<(), Box<dyn std::error::Error>> {
  // Clean up any existing test file
  let test_file = PathBuf::from("test_output.json");
  if test_file.exists() {
    fs::remove_file(&test_file)?;
  }

  println!("🚀 Starting Integration Workflow Test");
  println!("=====================================");
  TestContext::setup().await;

  // 1. Create node registry and register all required nodes
  let node_registry = NodeRegistry::new();

  let manual_trigger = ManualTriggerNode::new()?;
  node_registry.register_node(Arc::new(manual_trigger))?;

  let if_node = IfNode::new()?;
  node_registry.register_node(Arc::new(if_node))?;

  let set_node = EditFieldsNode::new()?;
  node_registry.register_node(Arc::new(set_node))?;

  let file_node = ReadWriteFilesNode::new()?;
  node_registry.register_node(Arc::new(file_node))?;

  println!("✅ Registered {} node types", node_registry.len());

  // 2. Create the integration workflow
  let workflow = create_integration_workflow()?;
  println!("✅ Created workflow with {} nodes", workflow.nodes.len());

  // 3. Create execution store and engine
  let workflow_engine: WorkflowEngineService = Application::global().component();
  println!("✅ Created DefaultWorkflowEngine");

  // 4. Create execution context
  let execution_id = ExecutionId::now_v7();
  let workflow_arc = Arc::new(workflow);
  let ctx = Ctx::try_new(CtxPayload::default(), Some(std::time::SystemTime::now()), Some(Uuid::now_v7().to_string()))?;

  let execution_context = ExecutionContext::new(execution_id.clone(), workflow_arc, ctx);
  println!("✅ Created execution context");

  // 5. Prepare trigger data
  let trigger_node_name = NodeName::from("manual_trigger");
  let initial_data = ExecutionData::new_json(
    json!([{
      "trigger_type": "manual",
      "execution_mode": "test",
      "timestamp": now().timestamp(),
      "trigger_id": Uuid::new_v4().to_string(),
      "message": "Integration test workflow started",
      "enabled": true,
      "test_data": {
        "input_value": 42,
        "input_text": "Hello, Integration Test!"
      }
    }]),
    None,
  );

  let mut trigger_data_map = ExecutionDataMap::default();
  trigger_data_map.insert(ConnectionKind::Main, vec![ExecutionDataItems::Items(vec![initial_data])]);

  let trigger_data = WorkflowTriggerData::normal(trigger_node_name, trigger_data_map);
  println!("✅ Prepared trigger data");

  // 6. Execute the workflow
  println!("\n🔄 Executing integration workflow...");
  println!("   Execution ID: {}", execution_id);

  let result = workflow_engine.execute_workflow(trigger_data, &execution_context).await;

  match result {
    Ok(execution_result) => {
      println!("✅ Workflow execution completed successfully!");
      println!("   Final Status: {:?}", execution_result.status);
      println!("   Duration: {}ms", execution_result.duration_ms);
      println!("   Nodes executed: {}", execution_result.nodes_result.len());
      println!("   End nodes: {:?}", execution_result.end_nodes);

      // Verify all nodes executed
      assert_eq!(execution_result.nodes_result.len(), 5); // All 5 nodes should execute

      // Check for any failed nodes
      let failed_nodes: Vec<_> = execution_result
        .nodes_result
        .iter()
        .filter(|(_, result)| result.status == NodeExecutionStatus::Failed)
        .collect();

      if !failed_nodes.is_empty() {
        println!("\n❌ 发现失败的节点:");
        for (node_name, result) in &failed_nodes {
          println!("   - '{}': {:?}", node_name, result.status);
          if let Some(error) = &result.error {
            println!("     错误: {}", error);
          }
        }
      }

      assert_eq!(
        execution_result.status,
        ExecutionStatus::Success,
        "工作流执行失败！失败的节点: {:?}",
        failed_nodes.iter().map(|(name, _)| name).collect::<Vec<_>>()
      );

      // 验证所有节点都成功执行
      assert!(
        failed_nodes.is_empty(),
        "存在 {} 个失败的节点: {:?}",
        failed_nodes.len(),
        failed_nodes.iter().map(|(name, _)| name).collect::<Vec<_>>()
      );

      // Print node execution details
      println!("\n📊 Node Execution Results:");
      for (node_name, node_result) in &execution_result.nodes_result {
        println!("   Node '{}': {:?} ({}ms)", node_name, node_result.status, node_result.duration_ms);

        if let Some(error) = &node_result.error {
          println!("     Error: {}", error);
        }

        // Print output data summary
        for (conn_kind, outputs) in &node_result.output_data {
          if let Some(ExecutionDataItems::Items(items)) = outputs.first() {
            println!("     Output {:?}: {} items", conn_kind, items.len());

            // Print some sample data for key nodes
            if (node_name.as_ref() == "condition_check" || node_name.as_ref().contains("set_data")) && !items.is_empty()
            {
              println!("       Sample data: {}", items[0].json());
            }
          }
        }
      }

      // Verify the workflow took the expected path (true branch since execution_mode is "test")
      let set_true_result = execution_result.nodes_result.get(&NodeName::from("set_data_true"));
      let set_false_result = execution_result.nodes_result.get(&NodeName::from("set_data_false"));

      assert!(set_true_result.is_some(), "True branch should execute");
      assert!(set_false_result.is_some(), "False branch should also be defined");

      // Check that true branch succeeded
      assert_eq!(set_true_result.unwrap().status, NodeExecutionStatus::Success);

      // Show final output data
      println!("\n📤 Final Output Data:");
      for end_node in &execution_result.end_nodes {
        if let Some(node_result) = execution_result.nodes_result.get(end_node) {
          for (conn_kind, outputs) in &node_result.output_data {
            if let Some(ExecutionDataItems::Items(items)) = outputs.first() {
              println!("   Node '{}' - {:?}:", end_node, conn_kind);
              for (i, item) in items.iter().enumerate() {
                println!("     Item {}: {}", i + 1, item.json());
              }
            }
          }
        }
      }

      // Verify file was created (if file operations were included)
      if test_file.exists() {
        println!("\n📁 File Operations:");
        println!("   ✅ Test file created: {:?}", test_file);

        // Read and display file content
        let content = fs::read_to_string(&test_file)?;
        println!("   File content: {}", content);

        // Clean up test file
        fs::remove_file(&test_file)?;
        println!("   ✅ Test file cleaned up");
      }

      println!("\n🎉 Integration test completed successfully!");
    }
    Err(e) => {
      println!("❌ Workflow execution failed: {}", e);
      return Err(e.into());
    }
  }

  Ok(())
}

#[tokio::test]
async fn test_integration_workflow_false_branch() -> Result<(), Box<dyn std::error::Error>> {
  // Test the false branch by changing the trigger data
  println!("\n🔄 Testing False Branch...");
  TestContext::setup().await;

  // Setup similar to main test but with different trigger data
  let node_registry = NodeRegistry::new();

  let manual_trigger = ManualTriggerNode::new()?;
  node_registry.register_node(Arc::new(manual_trigger))?;

  let if_node = IfNode::new()?;
  node_registry.register_node(Arc::new(if_node))?;

  let set_node = EditFieldsNode::new()?;
  node_registry.register_node(Arc::new(set_node))?;

  let file_node = ReadWriteFilesNode::new()?;
  node_registry.register_node(Arc::new(file_node))?;

  let workflow_engine: WorkflowEngineService = Application::global().component();

  let execution_id = ExecutionId::now_v7();
  let workflow_arc = Arc::new(create_integration_workflow()?);
  let ctx = Ctx::try_new(CtxPayload::default(), Some(std::time::SystemTime::now()), Some(Uuid::now_v7().to_string()))?;

  let execution_context = ExecutionContext::new(execution_id.clone(), workflow_arc, ctx);

  // Trigger data with execution_mode = "production" to trigger false branch
  let trigger_node_name = NodeName::from("manual_trigger");
  let initial_data = ExecutionData::new_json(
    json!([{
      "trigger_type": "manual",
      "execution_mode": "production", // This will trigger the false branch
      "timestamp": now().timestamp(),
      "trigger_id": Uuid::now_v7().to_string(),
      "message": "False branch test workflow started",
      "enabled": true,
    }]),
    None,
  );

  let mut trigger_data_map = ExecutionDataMap::default();
  trigger_data_map.insert(ConnectionKind::Main, vec![ExecutionDataItems::Items(vec![initial_data])]);

  let trigger_data = WorkflowTriggerData::normal(trigger_node_name, trigger_data_map);

  let result = workflow_engine.execute_workflow(trigger_data, &execution_context).await?;

  // Check for any failed nodes
  let failed_nodes: Vec<_> = result
    .nodes_result
    .iter()
    .filter(|(_, node_result)| node_result.status == NodeExecutionStatus::Failed)
    .collect();

  if !failed_nodes.is_empty() {
    println!("\n❌ False Branch 测试中发现失败的节点:");
    for (node_name, node_result) in &failed_nodes {
      println!("   - '{}': {:?}", node_name, node_result.status);
      if let Some(error) = &node_result.error {
        println!("     错误: {}", error);
      }
    }
  }

  assert_eq!(
    result.status,
    ExecutionStatus::Success,
    "False Branch 工作流执行失败！失败的节点: {:?}",
    failed_nodes.iter().map(|(name, _)| name).collect::<Vec<_>>()
  );

  // 验证所有节点都成功执行
  assert!(
    failed_nodes.is_empty(),
    "False Branch 测试中存在 {} 个失败的节点: {:?}",
    failed_nodes.len(),
    failed_nodes.iter().map(|(name, _)| name).collect::<Vec<_>>()
  );

  // Verify false branch was executed
  let set_false_result = result.nodes_result.get(&NodeName::from("set_data_false"));
  assert!(set_false_result.is_some());
  assert_eq!(set_false_result.unwrap().status, NodeExecutionStatus::Success);

  println!("✅ False branch test completed successfully!");

  Ok(())
}
