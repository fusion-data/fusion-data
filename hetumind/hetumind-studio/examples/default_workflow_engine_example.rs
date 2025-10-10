//! # DefaultWorkflowEngine Example
//!
//! This example demonstrates how to use the DefaultWorkflowEngine to execute a Workflow.
//! It shows how to:
//! - Create a simple workflow with multiple nodes
//! - Register node executors
//! - Execute the workflow and handle results
//! - Mock the ExecutionStore for demonstration

use std::sync::Arc;

use async_trait::async_trait;
use fusion_common::ahash::HashMap;
use fusion_common::ctx::{Ctx, CtxPayload};
use fusion_common::time::now;
use hetumind_core::workflow::{Execution, PinData, WorkflowExecutionError, WorkflowMeta, WorkflowNode, WorkflowStatus};
use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{
    Connection, ConnectionKind, ExecutionContext, ExecutionData, ExecutionDataItems, ExecutionDataMap, ExecutionId,
    ExecutionStatus, Node, NodeDefinition, NodeDefinitionBuilder, NodeExecutable, NodeExecutionContext,
    NodeExecutionError, NodeGroupKind, NodeKind, NodeName, NodeRegistry, ParameterMap, Workflow, WorkflowEngine,
    WorkflowEngineSetting, WorkflowId, WorkflowSettings,
  },
};
use hetumind_nodes::constants::START_TRIGGER_NODE_KIND;
use hetumind_nodes::trigger::StartNode;
use hetumind_studio::runtime::{
  checkpoint::{CheckpointError, ExecutionCheckpoint},
  execution::ExecutionStore,
  workflow::DefaultWorkflowEngine,
};
use mea::rwlock::RwLock;
use serde_json::json;

// Mock ExecutionStore for demonstration
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

// Simple data processing node for demonstration
pub struct SimpleProcessNodeV1 {
  definition: Arc<NodeDefinition>,
}
impl Default for SimpleProcessNodeV1 {
  fn default() -> Self {
    Self::new()
  }
}
impl SimpleProcessNodeV1 {
  pub fn new() -> Self {
    let definition = NodeDefinitionBuilder::default()
      .kind(NodeKind::from("SimpleProcess"))
      .version(Version::new(1, 0, 0))
      .groups(vec![NodeGroupKind::Transform])
      .display_name("Simple Process")
      .description("A simple data processing node that demonstrates basic workflow execution")
      .build()
      .unwrap();

    Self { definition: Arc::new(definition) }
  }
}

#[async_trait]
impl hetumind_core::workflow::NodeExecutable for SimpleProcessNodeV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // Get input data
    let input_data = &context.input_data;

    // Process the data - create a simple transformation
    let mut output_data = ExecutionDataMap::default();

    if let Some(main_input) = input_data.get(&ConnectionKind::Main) {
      if let Some(ExecutionDataItems::Items(items)) = main_input.first() {
        // Transform each input item
        let transformed_items: Vec<ExecutionData> = items
          .iter()
          .map(|item| {
            let input_json = item.json();
            let output_json = if input_json.is_object() {
              let mut obj = input_json.as_object().unwrap().clone();
              // Add processed timestamp
              obj.insert("processed_at".to_string(), JsonValue::String(now().to_rfc3339()));
              obj.insert("processor".to_string(), JsonValue::String("SimpleProcessNode".to_string()));
              JsonValue::Object(obj)
            } else {
              // If not an object, wrap it
              let mut obj = serde_json::Map::new();
              obj.insert("original_data".to_string(), input_json.clone());
              obj.insert("processed_at".to_string(), JsonValue::String(now().to_rfc3339()));
              obj.insert("processor".to_string(), JsonValue::String("SimpleProcessNode".to_string()));
              JsonValue::Object(obj)
            };

            ExecutionData::new_json(output_json, None)
          })
          .collect();

        output_data.insert(ConnectionKind::Main, vec![ExecutionDataItems::Items(transformed_items)]);
      } else {
        // Handle empty input
        let empty_item = ExecutionData::new_json(JsonValue::Object(serde_json::Map::new()), None);
        output_data.insert(ConnectionKind::Main, vec![ExecutionDataItems::Items(vec![empty_item])]);
      }
    } else {
      // No input data, create default output
      let default_item = ExecutionData::new_json(JsonValue::Object(serde_json::Map::new()), None);
      output_data.insert(ConnectionKind::Main, vec![ExecutionDataItems::Items(vec![default_item])]);
    }

    Ok(output_data)
  }
}

// Simple Node implementation for the process node
pub struct SimpleProcessNode {
  default_version: Version,
  executors: Vec<Arc<dyn NodeExecutable + Send + Sync>>,
}

impl Node for SimpleProcessNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[Arc<dyn NodeExecutable + Send + Sync>] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

impl SimpleProcessNode {
  pub fn new() -> Result<Self, hetumind_core::workflow::RegistrationError> {
    let executor = Arc::new(SimpleProcessNodeV1::new());
    let default_version = executor.definition().version.clone();
    let executors = vec![executor as Arc<dyn NodeExecutable + Send + Sync>];

    Ok(Self { default_version, executors })
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("🚀 DefaultWorkflowEngine Example");
  println!("=================================");

  // 1. Create a NodeRegistry
  let node_registry = NodeRegistry::new();

  // 2. Register the Start node
  let start_node = StartNode::new()?;
  node_registry.register_node(Arc::new(start_node))?;

  // 3. Register our custom process node
  let process_node = SimpleProcessNode::new()?;
  node_registry.register_node(Arc::new(process_node))?;

  println!("✅ Registered {} node types", node_registry.len());

  // 4. Create a simple workflow
  let workflow = create_sample_workflow()?;
  println!("✅ Created workflow with {} nodes", workflow.nodes.len());

  // 5. Create the execution store and config
  let execution_store = Arc::new(MockExecutionStore::default());
  let execution_config = WorkflowEngineSetting::default();

  // 6. Create the workflow engine
  let workflow_engine = DefaultWorkflowEngine::new(node_registry, execution_store, execution_config);
  println!("✅ Created DefaultWorkflowEngine");

  // 7. Create execution context
  let execution_id = ExecutionId::now_v7();
  let workflow_arc = Arc::new(workflow);
  let ctx =
    Ctx::try_new(CtxPayload::default(), Some(std::time::SystemTime::now()), Some(uuid::Uuid::now_v7().to_string()))?;

  let execution_context = ExecutionContext::new(execution_id.clone(), workflow_arc, ctx);
  println!("✅ Created execution context");

  // 8. Prepare trigger data (data that starts the workflow)
  let trigger_node_name = NodeName::from("start_node");
  let initial_data = ExecutionData::new_json(
    json!([{
      "message": "Hello, World!",
      "timestamp": now().to_rfc3339()
    }]),
    None,
  );

  let mut trigger_data_map = ExecutionDataMap::default();
  trigger_data_map.insert(ConnectionKind::Main, vec![ExecutionDataItems::Items(vec![initial_data])]);

  let trigger_data = (trigger_node_name, trigger_data_map);
  println!("✅ Prepared trigger data");

  // 9. Execute the workflow
  println!("\n🔄 Executing workflow...");
  println!("   Execution ID: {}", execution_id);

  let result = workflow_engine.execute_workflow(trigger_data, &execution_context).await;

  match result {
    Ok(execution_result) => {
      println!("✅ Workflow execution completed successfully!");
      println!("   Final Status: {:?}", execution_result.status);
      println!("   Duration: {}ms", execution_result.duration_ms);
      println!("   Nodes executed: {}", execution_result.nodes_result.len());
      println!("   End nodes: {:?}", execution_result.end_nodes);

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
          }
        }
      }

      // Show final output data
      println!("\n📤 Final Output Data:");
      for end_node in &execution_result.end_nodes {
        if let Some(node_result) = execution_result.nodes_result.get(end_node) {
          for (conn_kind, outputs) in &node_result.output_data {
            if let Some(ExecutionDataItems::Items(items)) = outputs.first() {
              println!("   Node '{}' - {:?}:", end_node, conn_kind);
              for (i, item) in items.iter().take(3).enumerate() {
                println!("     Item {}: {}", i + 1, item.json());
              }
              if items.len() > 3 {
                println!("     ... and {} more items", items.len() - 3);
              }
            }
          }
        }
      }
    }
    Err(e) => {
      println!("❌ Workflow execution failed: {}", e);
      return Err(e.into());
    }
  }

  println!("\n🎉 Example completed successfully!");
  Ok(())
}

fn create_sample_workflow() -> Result<Workflow, Box<dyn std::error::Error>> {
  let workflow_id = WorkflowId::now_v7();

  // Create nodes
  let start_node = WorkflowNode::builder()
    .name(NodeName::from("start_node"))
    .kind(NodeKind::from(START_TRIGGER_NODE_KIND))
    .display_name("Start")
    .parameters(ParameterMap::default())
    .build();

  let process_node = WorkflowNode::builder()
    .name(NodeName::from("process_node"))
    .kind(NodeKind::from("SimpleProcess"))
    .display_name("Process Data")
    .parameters(ParameterMap::default())
    .build();

  // Create connections
  let mut connections = HashMap::default();

  // Connect start_node to process_node
  let start_connections = vec![Connection::new(NodeName::from("process_node"), ConnectionKind::Main, 0)];
  let mut start_connection_map = HashMap::default();
  start_connection_map.insert(ConnectionKind::Main, start_connections);
  connections.insert(NodeName::from("start_node"), start_connection_map);

  let workflow = Workflow {
    id: workflow_id,
    name: "Example Workflow".to_string(),
    status: WorkflowStatus::Active,
    version: None,
    settings: WorkflowSettings::default(),
    meta: WorkflowMeta::default(),
    nodes: vec![start_node, process_node],
    connections,
    pin_data: PinData::default(),
    static_data: None,
  };

  Ok(workflow)
}
