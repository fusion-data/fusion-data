use hetumind_core::{
  types::JsonValue,
  version::Version,
  workflow::{NodeDefinition, NodeGroupKind, NodeProperty, NodePropertyKind},
};
use serde_json::json;

use crate::constants::SCHEDULE_TRIGGER_NODE_KIND;

pub fn create_base() -> NodeDefinition {
  NodeDefinition::new(SCHEDULE_TRIGGER_NODE_KIND, "Schedule Trigger")
    .add_group(NodeGroupKind::Trigger)
    .with_description("Triggers workflow execution on a scheduled basis")
    .add_property(
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("Schedule Mode")
        .with_name("schedule_mode")
        .with_options(vec![
          Box::new(NodeProperty::new_option(
            "Cron Expression",
            "cron",
            json!("cron"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Interval",
            "interval",
            json!("interval"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "Daily",
            "daily",
            json!("daily"),
            NodePropertyKind::String,
          )),
        ])
        .with_required(true)
        .with_description("The scheduling mode to use"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::String)
        .with_display_name("Cron Expression")
        .with_name("cron_expression")
        .with_required(false)
        .with_description("Standard cron expression (e.g., 0 */6 * * *)")
        .with_hint("支持标准 cron 语法：分 时 日 月 周")
        .with_placeholder("0 */6 * * *"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::String)
        .with_display_name("Interval")
        .with_name("interval")
        .with_required(false)
        .with_description("Predefined interval for scheduling (e.g., '30m', '1h', '2d')"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::String)
        .with_display_name("Daily Time")
        .with_name("daily_time")
        .with_required(false)
        .with_description("Time of day to trigger (HH:MM:ss format)")
        .with_hint("24小时制格式 (HH:MM:ss)")
        .with_placeholder("13:30:00"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("Timezone")
        .with_name("timezone")
        .with_required(false)
        .with_options(vec![
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
        .with_description("Timezone for scheduling"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::String)
        .with_display_name("Start Delay")
        .with_name("start_delay")
        .with_required(false)
        .with_description("Delay before first execution (e.g., '30s', '1m', '2h')"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Number)
        .with_display_name("Max Executions")
        .with_name("max_executions")
        .with_required(false)
        .with_description("Maximum number of executions (0 = unlimited)")
        .with_value(json!(0)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Number)
        .with_display_name("Retry Count")
        .with_name("retry_count")
        .with_required(false)
        .with_description("Number of retries on failure (0 = no retry)")
        .with_value(json!(3)),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Options)
        .with_display_name("Retry Interval")
        .with_name("retry_interval")
        .with_required(false)
        .with_options(vec![
          Box::new(NodeProperty::new_option(
            "1 Minute",
            "1m",
            json!("1m"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "5 Minutes",
            "5m",
            json!("5m"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "15 Minutes",
            "15m",
            json!("15m"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "30 Minutes",
            "30m",
            json!("30m"),
            NodePropertyKind::String,
          )),
          Box::new(NodeProperty::new_option(
            "1 Hour",
            "1h",
            json!("1h"),
            NodePropertyKind::String,
          )),
        ])
        .with_description("Interval between retry attempts"),
    )
    .add_property(
      NodeProperty::new(NodePropertyKind::Boolean)
        .with_display_name("Enabled")
        .with_name("enabled")
        .with_required(false)
        .with_description("Whether the schedule trigger is enabled")
        .with_value(json!(true)),
    )
}
