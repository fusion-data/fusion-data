//! Schedule trigger configuration parsing utilities
use std::str::FromStr;
use std::time::Duration;

use croner::Cron;
use duration_str::deserialize_option_duration;
use hetumind_core::workflow::{NodeExecutionError, ParameterMap, ValidationError};
use serde::{Deserialize, Serialize};

/// Schedule configuration parsed from node parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleParameters {
  pub mode: ScheduleMode,

  pub cron_expression: Option<String>,

  #[serde(deserialize_with = "deserialize_option_duration")]
  pub interval: Option<Duration>,

  #[serde(deserialize_with = "deserialize_option_duration")]
  pub start_delay: Option<Duration>,

  pub timezone: Option<String>,

  pub max_executions: Option<u32>,

  pub retry_count: Option<u32>,

  #[serde(deserialize_with = "deserialize_option_duration")]
  pub retry_interval: Option<Duration>,
}

/// Scheduling mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleMode {
  Cron,
  Interval,
}

/// Parse schedule configuration from node parameters
pub fn parse_schedule_parameteres(parameters: &ParameterMap) -> Result<ScheduleParameters, NodeExecutionError> {
  let config: ScheduleParameters = parameters.get()?;
  if config.mode == ScheduleMode::Cron {
    if let Some(cron_expression) = config.cron_expression.as_deref() {
      Cron::from_str(cron_expression).map_err(|e| {
        NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
          "cron_expression",
          format!("Invalid cron expression: {}", e),
        ))
      })?;
    } else {
      return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
        "cron_expression",
        "Cron expression is required for cron mode",
      )));
    }
  }
  if config.mode == ScheduleMode::Interval && config.interval.is_none() {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "interval",
      "Interval is required for interval mode",
    )));
  }
  Ok(config)
}
