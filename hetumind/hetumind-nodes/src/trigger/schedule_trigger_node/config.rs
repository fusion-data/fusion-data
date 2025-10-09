//! Schedule trigger configuration parsing utilities

use hetumind_core::types::JsonValue;
use hetumind_core::workflow::{NodeExecutionError, ValidationError};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::time::Duration;

/// Schedule configuration parsed from node parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
  pub mode: ScheduleMode,
  pub cron_expression: Option<String>,
  pub interval: Option<String>,
  pub custom_interval: Option<String>,
  pub daily_time: Option<String>,
  pub timezone: String,
  pub start_delay: u64,
  pub max_executions: u32,
  pub retry_count: u32,
  pub retry_interval: String,
  pub enabled: bool,
}

/// Scheduling mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScheduleMode {
  Cron,
  Interval,
  Daily,
}

/// Parse schedule configuration from node parameters
pub fn parse_schedule_config(parameters: &Map<String, JsonValue>) -> Result<ScheduleConfig, NodeExecutionError> {
  let mode = parameters
    .get("schedule_mode")
    .and_then(|v| v.as_str())
    .ok_or_else(|| NodeExecutionError::ParameterValidation(ValidationError::required_field_missing("schedule_mode")))?;

  let schedule_mode = match mode {
    "cron" => ScheduleMode::Cron,
    "interval" => ScheduleMode::Interval,
    "daily" => ScheduleMode::Daily,
    _ => {
      return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
        "scheduleMode",
        format!("Invalid schedule mode: {}", mode),
      )));
    }
  };

  // Validate required fields for each mode
  match schedule_mode {
    ScheduleMode::Cron => {
      if parameters.get("cron_expression").and_then(|v| v.as_str()).map_or(true, |s| s.trim().is_empty()) {
        return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
          "cron_expression",
          "Cron expression is required for cron mode",
        )));
      }
    }
    ScheduleMode::Interval => {
      let has_interval = parameters.get("interval").and_then(|v| v.as_str()).map_or(false, |s| !s.trim().is_empty());
      let has_custom =
        parameters.get("custom_interval").and_then(|v| v.as_str()).map_or(false, |s| !s.trim().is_empty());
      if !has_interval && !has_custom {
        return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
          "interval",
          "Interval or custom interval is required for interval mode",
        )));
      }
    }
    ScheduleMode::Daily => {
      if parameters.get("daily_time").and_then(|v| v.as_str()).map_or(true, |s| s.trim().is_empty()) {
        return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
          "daily_time",
          "Daily time is required for daily mode",
        )));
      }
    }
  }

  Ok(ScheduleConfig {
    mode: schedule_mode,
    cron_expression: parameters.get("cron_expression").and_then(|v| v.as_str()).map(|s| s.trim().to_string()),
    interval: parameters.get("interval").and_then(|v| v.as_str()).map(|s| s.trim().to_string()),
    custom_interval: parameters.get("custom_interval").and_then(|v| v.as_str()).map(|s| s.trim().to_string()),
    daily_time: parameters.get("daily_time").and_then(|v| v.as_str()).map(|s| s.trim().to_string()),
    timezone: parameters.get("timezone").and_then(|v| v.as_str()).unwrap_or("UTC").to_string(),
    start_delay: parameters.get("start_delay").and_then(|v| v.as_u64()).unwrap_or(0),
    max_executions: parameters.get("max_executions").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
    retry_count: parameters.get("retry_count").and_then(|v| v.as_u64()).unwrap_or(3) as u32,
    retry_interval: parameters.get("retry_interval").and_then(|v| v.as_str()).unwrap_or("5m").to_string(),
    enabled: parameters.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
  })
}

/// Parse duration string to Duration
/// Examples: "30s", "5m", "2h", "1d"
pub fn parse_duration(duration_str: &str) -> Result<Duration, NodeExecutionError> {
  let trimmed = duration_str.trim();
  if trimmed.is_empty() {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "duration",
      "Duration cannot be empty",
    )));
  }

  let regex = Regex::new(r"^(\d+)([smhd])$").map_err(|e| {
    NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "duration",
      format!("Invalid duration regex: {}", e),
    ))
  })?;

  let captures = regex.captures(trimmed).ok_or_else(|| {
    NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "duration",
      format!("Invalid duration format: {}. Expected format: number + unit (s/m/h/d)", duration_str),
    ))
  })?;

  let value: u64 = captures[1].parse().map_err(|_| {
    NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "duration",
      format!("Invalid duration value: {}", &captures[1]),
    ))
  })?;

  let unit = &captures[2];
  let duration = match unit {
    "s" => Duration::from_secs(value),
    "m" => Duration::from_secs(value * 60),
    "h" => Duration::from_secs(value * 3600),
    "d" => Duration::from_secs(value * 86400),
    _ => {
      return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
        "duration",
        format!("Invalid duration unit: {}", unit),
      )));
    }
  };

  Ok(duration)
}

/// Parse time string (HH:MM) to hour and minute
pub fn parse_time(time_str: &str) -> Result<(u32, u32), NodeExecutionError> {
  let trimmed = time_str.trim();
  if trimmed.is_empty() {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "time",
      "Time cannot be empty",
    )));
  }

  let parts: Vec<&str> = trimmed.split(':').collect();
  if parts.len() != 2 {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "time",
      format!("Invalid time format: {}. Expected HH:MM", time_str),
    )));
  }

  let hour: u32 = parts[0].parse().map_err(|_| {
    NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "time",
      format!("Invalid hour: {}", parts[0]),
    ))
  })?;
  let minute: u32 = parts[1].parse().map_err(|_| {
    NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "time",
      format!("Invalid minute: {}", parts[1]),
    ))
  })?;

  if hour > 23 {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "time",
      format!("Hour must be between 0 and 23: {}", hour),
    )));
  }

  if minute > 59 {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "time",
      format!("Minute must be between 0 and 59: {}", minute),
    )));
  }

  Ok((hour, minute))
}

/// Validate cron expression format
pub fn validate_cron_expression(cron_expr: &str) -> Result<(), NodeExecutionError> {
  let trimmed = cron_expr.trim();
  if trimmed.is_empty() {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "cronExpression",
      "Cron expression cannot be empty",
    )));
  }

  // Basic validation for cron format (5 fields: minute hour day month weekday)
  let parts: Vec<&str> = trimmed.split_whitespace().collect();
  if parts.len() != 5 {
    return Err(NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
      "cronExpression",
      "Cron expression must have 5 fields: minute hour day month weekday",
    )));
  }

  // Validate each field
  let valid_ranges = [(0, 59), (0, 23), (1, 31), (1, 12), (0, 6)];
  for (i, part) in parts.iter().enumerate() {
    validate_cron_field(part, valid_ranges[i].0, valid_ranges[i].1).map_err(|e| {
      NodeExecutionError::ParameterValidation(ValidationError::invalid_field_value(
        "cronExpression",
        format!("Invalid cron field {}: {}", i + 1, e),
      ))
    })?;
  }

  Ok(())
}

/// Validate individual cron field
fn validate_cron_field(field: &str, min: u32, max: u32) -> Result<(), String> {
  if field == "*" {
    return Ok(());
  }

  // Handle ranges (e.g., 1-5)
  if field.contains('-') {
    let parts: Vec<&str> = field.split('-').collect();
    if parts.len() != 2 {
      return Err("Invalid range format".to_string());
    }
    let start: u32 = parts[0].parse().map_err(|_| "Invalid range start".to_string())?;
    let end: u32 = parts[1].parse().map_err(|_| "Invalid range end".to_string())?;
    if start < min || end > max || start > end {
      return Err(format!("Range must be between {} and {}", min, max));
    }
    return Ok(());
  }

  // Handle step values (e.g., */5 or 1-10/2)
  if field.contains('/') {
    let parts: Vec<&str> = field.split('/').collect();
    if parts.len() != 2 {
      return Err("Invalid step format".to_string());
    }
    validate_cron_field(parts[0], min, max)?;
    let step: u32 = parts[1].parse().map_err(|_| "Invalid step value".to_string())?;
    if step == 0 {
      return Err("Step value must be greater than 0".to_string());
    }
    return Ok(());
  }

  // Handle comma-separated values (e.g., 1,2,5)
  if field.contains(',') {
    let values: Vec<&str> = field.split(',').collect();
    for value in values {
      validate_cron_field(value, min, max)?;
    }
    return Ok(());
  }

  // Handle single value
  let value: u32 = field.parse().map_err(|_| "Invalid number".to_string())?;
  if value < min || value > max {
    return Err(format!("Value must be between {} and {}", min, max));
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_duration() {
    assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
    assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
    assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
    assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86400));

    assert!(parse_duration("").is_err());
    assert!(parse_duration("30x").is_err());
    assert!(parse_duration("abc").is_err());
  }

  #[test]
  fn test_parse_time() {
    assert_eq!(parse_time("13:30").unwrap(), (13, 30));
    assert_eq!(parse_time("00:00").unwrap(), (0, 0));
    assert_eq!(parse_time("23:59").unwrap(), (23, 59));

    assert!(parse_time("").is_err());
    assert!(parse_time("25:00").is_err());
    assert!(parse_time("12:60").is_err());
    assert!(parse_time("abc").is_err());
  }

  #[test]
  fn test_validate_cron_expression() {
    assert!(validate_cron_expression("0 */6 * * *").is_ok());
    assert!(validate_cron_expression("*/30 * * * *").is_ok());
    assert!(validate_cron_expression("0 0 * * *").is_ok());
    assert!(validate_cron_expression("0 9 * * 1-5").is_ok());

    assert!(validate_cron_expression("").is_err());
    assert!(validate_cron_expression("0 * *").is_err()); // Missing fields
    assert!(validate_cron_expression("60 * * * *").is_err()); // Invalid minute
    assert!(validate_cron_expression("* 25 * * *").is_err()); // Invalid hour
  }
}
