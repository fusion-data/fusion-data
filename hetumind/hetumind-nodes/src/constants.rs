// Core Nodes
pub static AGGREGATE_NODE_KIND: &str = "hetumind_nodes::Aggregate";
pub static IF_NODE_KIND: &str = "hetumind_nodes::If";
pub static LIMIT_NODE_KIND: &str = "hetumind_nodes::Limit";
pub static LOOP_OVER_ITEMS_NODE_KIND: &str = "hetumind_nodes::LoopOverItems";
pub static MERGE_NODE_KIND: &str = "hetumind_nodes::Merge";
pub static NOOP_NODE_KIND: &str = "hetumind_nodes::NoOp";
pub static READ_WRITE_FILES_NODE_KIND: &str = "hetumind_nodes::ReadWriteFiles";
pub static SET_NODE_KIND: &str = "hetumind_nodes::Set";
pub static SPLIT_OUT_NODE_KIND: &str = "hetumind_nodes::SplitOut";
pub static STOP_AND_ERROR_NODE_KIND: &str = "hetumind_nodes::StopAndError";
pub static SUMMARIZE_NODE_KIND: &str = "hetumind_nodes::Summarize";
pub static SWITCH_NODE_KIND: &str = "hetumind_nodes::Switch";

// Trigger Nodes
pub static SCHEDULE_TRIGGER_NODE_KIND: &str = "hetumind_nodes::ScheduleTrigger";
pub static WEBHOOK_TRIGGER_NODE_KIND: &str = "hetumind_nodes::WebhookTrigger";
pub static START_TRIGGER_NODE_KIND: &str = "hetumind_nodes::Start";
pub static CHAT_TRIGGERN_NODE_KIND: &str = "hetumind_nodes::ChatTrigger";
pub static ERROR_TRIGGER_NODE_KIND: &str = "hetumind_nodes::ErrorTrigger";
pub static MANUAL_TRIGGER_NODE_KIND: &str = "hetumind_nodes::ManualTrigger";

// Integration Nodes
pub static HTTP_REQUEST_NODE_KIND: &str = "hetumind_nodes::HttpRequest";
