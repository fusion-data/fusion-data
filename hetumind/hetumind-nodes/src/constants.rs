// Core Nodes
pub static AGGREGATE_NODE_KIND: &str = "hetumind_nodes::Aggregate";
pub static COMPARE_DATASETS_NODE_KIND: &str = "hetumind_nodes::CompareDatasets";
pub static IF_NODE_KIND: &str = "hetumind_nodes::If";
pub static LIMIT_NODE_KIND: &str = "hetumind_nodes::Limit";
pub static LOOP_OVER_ITEMS_NODE_KIND: &str = "hetumind_nodes::LoopOverItems";
pub static MERGE_NODE_KIND: &str = "hetumind_nodes::Merge";
pub static NOOP_NODE_KIND: &str = "hetumind_nodes::NoOp";
pub static READ_WRITE_FILES_NODE_KIND: &str = "hetumind_nodes::ReadWriteFiles";
pub static EDIT_FIELDS_NODE_KIND: &str = "hetumind_nodes::EditFields";
pub static EDIT_IMAGE_NODE_KIND: &str = "hetumind_nodes::EditImage";
pub static SPLIT_OUT_NODE_KIND: &str = "hetumind_nodes::SplitOut";
pub static STOP_AND_ERROR_NODE_KIND: &str = "hetumind_nodes::StopAndError";
pub static SUMMARIZE_NODE_KIND: &str = "hetumind_nodes::Summarize";
pub static SWITCH_NODE_KIND: &str = "hetumind_nodes::Switch";
pub static WAIT_NODE_KIND: &str = "hetumind_nodes::Wait";

// Trigger Nodes
pub static SCHEDULE_TRIGGER_NODE_KIND: &str = "hetumind_nodes::ScheduleTrigger";
pub static WEBHOOK_TRIGGER_NODE_KIND: &str = "hetumind_nodes::WebhookTrigger";
pub static START_TRIGGER_NODE_KIND: &str = "hetumind_nodes::Start";
pub static CHAT_TRIGGERN_NODE_KIND: &str = "hetumind_nodes::ChatTrigger";
pub static ERROR_TRIGGER_NODE_KIND: &str = "hetumind_nodes::ErrorTrigger";
pub static MANUAL_TRIGGER_NODE_KIND: &str = "hetumind_nodes::ManualTrigger";
pub static EMAIL_TRIGGER_NODE_KIND: &str = "hetumind_nodes::EmailTrigger";

// Integration Nodes
pub static HTTP_REQUEST_NODE_KIND: &str = "hetumind_nodes::HttpRequest";
pub static SEND_EMAIL_NODE_KIND: &str = "hetumind_nodes::SendEmail";

// Root Nodes for Cluster Nodes
pub static AI_AGENT_NODE_KIND: &str = "hetumind_nodes::AiAgent";

// Sub-nodes for Cluster Nodes
pub static CHAT_MODEL_NODE_KIND: &str = "hetumind_nodes::ChatModel";
