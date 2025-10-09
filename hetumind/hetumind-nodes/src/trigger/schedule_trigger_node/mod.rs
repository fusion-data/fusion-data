//! # ScheduleTriggerNode
//!
//! A node that triggers workflow execution on a scheduled basis.

use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::types::JsonValue;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, Node, NodeDefinition, NodeDefinitionBuilder, NodeExecutable,
  NodeExecutionContext, NodeExecutionError, NodeExecutor, NodeGroupKind, NodeKind, NodeProperty, NodePropertyKind,
  RegistrationError, make_execution_data_map,
};

use crate::constants::SCHEDULE_TRIGGER_NODE_KIND;

pub mod config;

pub struct ScheduleTriggerNodeV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinitionBuilder> for ScheduleTriggerNodeV1 {
  type Error = RegistrationError;

  fn try_from(builder: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    let definition = builder.build()?;
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl NodeExecutable for ScheduleTriggerNodeV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, _context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    // Schedule 触发器作为入口点，返回空数据
    // 实际的调度逻辑在触发器框架层面完成
    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]))
  }
}

pub struct ScheduleTriggerNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl Node for ScheduleTriggerNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[NodeExecutor] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

impl ScheduleTriggerNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = create_base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(ScheduleTriggerNodeV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

fn create_base() -> NodeDefinitionBuilder {
  let mut base = NodeDefinitionBuilder::default();
  base
    .kind(SCHEDULE_TRIGGER_NODE_KIND)
    .version(Version::new(1, 0, 0))
    .groups([NodeGroupKind::Trigger])
    .display_name("Schedule Trigger")
    .description("Triggers workflow execution on a scheduled basis")
    .outputs(vec![])
    .properties(vec![
      // 调度模式选择
      NodeProperty::builder()
        .display_name("Schedule Mode")
        .name("schedule_mode")
        .kind(NodePropertyKind::Options)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "Cron Expression",
            "cron",
            JsonValue::String("cron".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Interval",
            "interval",
            JsonValue::String("interval".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Daily",
            "daily",
            JsonValue::String("daily".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .required(true)
        .description("The scheduling mode to use")
        .build(),
      // Cron 表达式
      NodeProperty::builder()
        .display_name("Cron Expression")
        .name("cron_expression")
        .kind(NodePropertyKind::String)
        .required(false)
        .description("Standard cron expression (e.g., 0 */6 * * *)")
        .hint("支持标准 cron 语法：分 时 日 月 周")
        .placeholder("0 */6 * * *")
        .build(),
      // 间隔模式
      NodeProperty::builder()
        .display_name("Interval")
        .name("interval")
        .kind(NodePropertyKind::Options)
        .required(false)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "Every 30 Seconds",
            "30s",
            JsonValue::String("30s".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Every Minute",
            "1m",
            JsonValue::String("1m".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Every 5 Minutes",
            "5m",
            JsonValue::String("5m".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Every 15 Minutes",
            "15m",
            JsonValue::String("15m".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Every 30 Minutes",
            "30m",
            JsonValue::String("30m".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Every Hour",
            "1h",
            JsonValue::String("1h".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Every 6 Hours",
            "6h",
            JsonValue::String("6h".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Every 12 Hours",
            "12h",
            JsonValue::String("12h".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Every Day",
            "1d",
            JsonValue::String("1d".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Custom",
            "custom",
            JsonValue::String("custom".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .description("Predefined interval for scheduling")
        .build(),
      // 自定义间隔
      NodeProperty::builder()
        .display_name("Custom Interval")
        .name("custom_interval")
        .kind(NodePropertyKind::String)
        .required(false)
        .description("Custom interval (e.g., 45s, 2h, 30m)")
        .hint("格式: 数字+单位 (s/m/h/d)")
        .placeholder("30s, 5m, 1h, 1d")
        .build(),
      // 每日时间
      NodeProperty::builder()
        .display_name("Daily Time")
        .name("daily_time")
        .kind(NodePropertyKind::String)
        .required(false)
        .description("Time of day to trigger (HH:MM format)")
        .hint("24小时制格式 (HH:MM)")
        .placeholder("13:30")
        .build(),
      // 时区
      NodeProperty::builder()
        .display_name("Timezone")
        .name("timezone")
        .kind(NodePropertyKind::Options)
        .required(false)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "UTC",
            "UTC",
            JsonValue::String("UTC".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Asia/Shanghai",
            "Asia/Shanghai",
            JsonValue::String("Asia/Shanghai".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "America/New_York",
            "America/New_York",
            JsonValue::String("America/New_York".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Europe/London",
            "Europe/London",
            JsonValue::String("Europe/London".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .description("Timezone for scheduling")
        .build(),
      // 启动延迟
      NodeProperty::builder()
        .display_name("Start Delay")
        .name("start_delay")
        .kind(NodePropertyKind::Number)
        .required(false)
        .description("Delay before first execution (seconds)")
        .value(JsonValue::Number(serde_json::Number::from(0)))
        .build(),
      // 最大执行次数
      NodeProperty::builder()
        .display_name("Max Executions")
        .name("max_executions")
        .kind(NodePropertyKind::Number)
        .required(false)
        .description("Maximum number of executions (0 = unlimited)")
        .value(JsonValue::Number(serde_json::Number::from(0)))
        .build(),
      // 错误重试次数
      NodeProperty::builder()
        .display_name("Retry Count")
        .name("retry_count")
        .kind(NodePropertyKind::Number)
        .required(false)
        .description("Number of retries on failure (0 = no retry)")
        .value(JsonValue::Number(serde_json::Number::from(3)))
        .build(),
      // 重试间隔
      NodeProperty::builder()
        .display_name("Retry Interval")
        .name("retry_interval")
        .kind(NodePropertyKind::Options)
        .required(false)
        .options(vec![
          Box::new(NodeProperty::new_option(
            "1 Minute",
            "1m",
            JsonValue::String("1m".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "5 Minutes",
            "5m",
            JsonValue::String("5m".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "15 Minutes",
            "15m",
            JsonValue::String("15m".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "30 Minutes",
            "30m",
            JsonValue::String("30m".to_string()),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "1 Hour",
            "1h",
            JsonValue::String("1h".to_string()),
            NodePropertyKind::String,
          )),
        ])
        .description("Interval between retry attempts")
        .build(),
      // 启用状态
      NodeProperty::builder()
        .display_name("Enabled")
        .name("enabled")
        .kind(NodePropertyKind::Boolean)
        .required(false)
        .description("Whether the schedule trigger is enabled")
        .value(JsonValue::Bool(true))
        .build(),
    ]);
  base
}
