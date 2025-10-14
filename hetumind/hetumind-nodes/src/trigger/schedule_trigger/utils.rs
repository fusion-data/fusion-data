use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{NodeDefinition, NodeGroupKind, NodeProperty, NodePropertyKind},
};
use serde_json::json;

use crate::constants::SCHEDULE_TRIGGER_NODE_KIND;

pub fn create_base() -> NodeDefinition {
  NodeDefinition::new(SCHEDULE_TRIGGER_NODE_KIND, Version::new(1, 0, 0), "Schedule Trigger")
    .add_group(NodeGroupKind::Trigger)
    .with_description("Triggers workflow execution on a scheduled basis")
    .add_property(
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
    )
    .add_property(
      NodeProperty::builder()
        .display_name("Cron Expression")
        .name("cron_expression")
        .kind(NodePropertyKind::String)
        .required(false)
        .description("Standard cron expression (e.g., 0 */6 * * *)")
        .hint("支持标准 cron 语法：分 时 日 月 周")
        .placeholder("0 */6 * * *")
        .build(),
    )
    .add_property(
      NodeProperty::builder()
        .display_name("Interval")
        .name("interval")
        .kind(NodePropertyKind::String)
        .required(false)
        .description("Predefined interval for scheduling (e.g., '30m', '1h', '2d')")
        .build(),
    )
    .add_property(
      NodeProperty::builder()
        .display_name("Daily Time")
        .name("daily_time")
        .kind(NodePropertyKind::String)
        .required(false)
        .description("Time of day to trigger (HH:MM:ss format)")
        .hint("24小时制格式 (HH:MM:ss)")
        .placeholder("13:30:00")
        .build(),
    )
    .add_property(
      NodeProperty::builder()
        .display_name("Timezone")
        .name("timezone")
        .kind(NodePropertyKind::Options)
        .required(false)
        .options(vec![
          Box::new(NodeProperty::new_option("UTC", "UTC", json!("UTC"), NodePropertyKind::String)),
          Box::new(NodeProperty::new_option(
            "Asia/Shanghai",
            "Asia/Shanghai",
            json!("Asia/Shanghai"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "America/New_York",
            "America/New_York",
            json!("America/New_York"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Europe/London",
            "Europe/London",
            json!("Europe/London"),
            NodePropertyKind::String,
          )),
        ])
        .description("Timezone for scheduling")
        .build(),
    )
    .add_property(
      NodeProperty::builder()
        .display_name("Start Delay")
        .name("start_delay")
        .kind(NodePropertyKind::String)
        .required(false)
        .description("Delay before first execution (e.g., '30s', '1m', '2h')")
        .build(),
    )
    .add_property(
      NodeProperty::builder()
        .display_name("Max Executions")
        .name("max_executions")
        .kind(NodePropertyKind::Number)
        .required(false)
        .description("Maximum number of executions (0 = unlimited)")
        .value(JsonValue::Number(serde_json::Number::from(0)))
        .build(),
    )
    .add_property(
      NodeProperty::builder()
        .display_name("Retry Count")
        .name("retry_count")
        .kind(NodePropertyKind::Number)
        .required(false)
        .description("Number of retries on failure (0 = no retry)")
        .value(JsonValue::Number(serde_json::Number::from(3)))
        .build(),
    )
    .add_property(
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
    )
    .add_property(
      NodeProperty::builder()
        .display_name("Enabled")
        .name("enabled")
        .kind(NodePropertyKind::Boolean)
        .required(false)
        .description("Whether the schedule trigger is enabled")
        .value(JsonValue::Bool(true))
        .build(),
    )
}
