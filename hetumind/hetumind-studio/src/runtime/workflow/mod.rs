mod default_workflow_engine;
mod engine_router;
mod workflow_engine_plugin;

use std::sync::Arc;

pub use default_workflow_engine::DefaultWorkflowEngine;
pub use engine_router::EngineRouter;
use hetumind_core::workflow::WorkflowEngine;
pub use workflow_engine_plugin::WorkflowEnginePlugin;

pub type WorkflowEngineService = Arc<dyn WorkflowEngine>;
