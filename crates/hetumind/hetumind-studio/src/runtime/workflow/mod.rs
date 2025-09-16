mod default_workflow_engine;
mod workflow_engine_plugin;

use std::sync::Arc;

pub use default_workflow_engine::DefaultWorkflowEngine;
use hetumind_core::workflow::WorkflowEngine;
pub use workflow_engine_plugin::WorkflowEnginePlugin;

pub type WorkflowEngineService = Arc<dyn WorkflowEngine>;
