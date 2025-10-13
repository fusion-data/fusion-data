//! Wait 等待节点实现
//!
//! 参考 n8n 的 Wait 节点设计，用于在工作流执行过程中添加等待机制。
//! 支持四种等待模式：时间间隔、特定时间、Webhook 触发和表单提交。

use chrono::{DateTime, Utc};
use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinitionBuilder, NodeExecutor, NodeGroupKind, NodeKind, RegistrationError, ValidationError},
};
use serde::{Deserialize, Serialize};

mod utils;
mod wait_v1;

use wait_v1::WaitV1;

use crate::constants::WAIT_NODE_KIND;

/// 等待模式类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitMode {
  /// 时间间隔等待
  TimeInterval,
  /// 特定时间等待
  SpecificTime,
  /// Webhook 触发等待
  Webhook,
  /// 表单提交等待
  Form,
}

/// 时间单位
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
  /// 秒
  Seconds,
  /// 分钟
  Minutes,
  /// 小时
  Hours,
  /// 天
  Days,
}

/// 响应模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseMode {
  /// 接收到响应时立即响应
  OnReceived,
  /// 最后一个节点完成后响应
  OnExecutionFinished,
  /// 等待 Webhook 调用时不响应
  Never,
}

/// HTTP 方法
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
  Get,
  Post,
  Put,
  Patch,
  Delete,
  Head,
}

/// 认证类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationType {
  /// 无认证
  None,
  /// 基础认证
  BasicAuth,
  /// Header 认证
  HeaderAuth,
  /// OAuth2
  OAuth2,
  /// Digest 认证
  DigestAuth,
}

/// 时间限制类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitType {
  /// 时间间隔后
  AfterTimeInterval,
  /// 特定时间
  AtSpecifiedTime,
}

/// Webhook 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
  /// HTTP 方法
  pub http_method: HttpMethod,
  /// 响应模式
  pub response_mode: ResponseMode,
  /// Webhook 后缀
  pub webhook_suffix: Option<String>,
  /// 认证类型
  pub authentication_type: Option<AuthenticationType>,
  /// 响应数据
  pub response_data: Option<serde_json::Value>,
  /// 响应头
  pub response_headers: Option<std::collections::HashMap<String, String>>,
  /// 响应状态码
  pub response_status_code: Option<u16>,
}

/// 表单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormConfig {
  /// 表单标题
  pub form_title: String,
  /// 表单描述
  pub form_description: Option<String>,
  /// 表单字段定义
  pub form_fields: Option<Vec<FormField>>,
  /// 重定向 URL
  pub redirect_url: Option<String>,
  /// 显示提交按钮
  pub show_submit_button: Option<bool>,
  /// 显示返回按钮
  pub show_back_button: Option<bool>,
  /// 按钮标签
  pub button_labels: Option<ButtonLabels>,
}

/// 表单字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
  /// 字段 ID
  pub field_id: String,
  /// 字段标签
  pub field_label: String,
  /// 字段类型
  pub field_type: FormFieldType,
  /// 是否必填
  pub required: Option<bool>,
  /// 默认值
  pub default_value: Option<serde_json::Value>,
  /// 选项（用于 select、radio 等）
  pub options: Option<Vec<FormOption>>,
  /// 验证规则
  pub validation: Option<FieldValidation>,
}

/// 表单字段类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormFieldType {
  /// 文本输入
  Text,
  /// 文本区域
  Textarea,
  /// 数字输入
  Number,
  /// 邮箱
  Email,
  /// 密码
  Password,
  /// 下拉选择
  Select,
  /// 单选按钮
  Radio,
  /// 复选框
  Checkbox,
  /// 日期
  Date,
  /// 时间
  Time,
  /// 文件上传
  File,
  /// 隐藏字段
  Hidden,
}

/// 表单选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormOption {
  /// 选项值
  pub value: String,
  /// 选项标签
  pub label: String,
  /// 是否选中
  pub selected: Option<bool>,
}

/// 字段验证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidation {
  /// 最小长度
  pub min_length: Option<usize>,
  /// 最大长度
  pub max_length: Option<usize>,
  /// 最小值
  pub min_value: Option<f64>,
  /// 最大值
  pub max_value: Option<f64>,
  /// 正则表达式
  pub pattern: Option<String>,
  /// 错误消息
  pub error_message: Option<String>,
}

/// 按钮标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonLabels {
  /// 提交按钮标签
  pub submit: Option<String>,
  /// 返回按钮标签
  pub back: Option<String>,
}

/// 时间限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLimitConfig {
  /// 是否启用时间限制
  pub enabled: bool,
  /// 限制类型
  pub limit_type: LimitType,
  /// 时间间隔量
  pub resume_amount: Option<u64>,
  /// 时间间隔单位
  pub resume_unit: Option<TimeUnit>,
  /// 最大日期时间
  pub max_date_and_time: Option<DateTime<Utc>>,
}

/// Wait 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitConfig {
  /// 等待模式
  pub wait_mode: WaitMode,
  /// 时间间隔配置
  pub time_interval: Option<TimeIntervalConfig>,
  /// 特定时间配置
  pub specific_time: Option<SpecificTimeConfig>,
  /// Webhook 配置
  pub webhook_config: Option<WebhookConfig>,
  /// 表单配置
  pub form_config: Option<FormConfig>,
  /// 时间限制配置
  pub time_limit: Option<TimeLimitConfig>,
}

/// 时间间隔配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeIntervalConfig {
  /// 等待时间量
  pub amount: u64,
  /// 时间单位
  pub unit: TimeUnit,
}

/// 特定时间配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificTimeConfig {
  /// 目标日期时间
  pub date_time: DateTime<Utc>,
  /// 时区（可选）
  pub timezone: Option<String>,
}

impl WaitConfig {
  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), ValidationError> {
    match self.wait_mode {
      WaitMode::TimeInterval => {
        if self.time_interval.is_none() {
          return Err(ValidationError::invalid_field_value(
            "time_interval".to_string(),
            "Time interval configuration is required for time interval mode".to_string(),
          ));
        }
        if let Some(ref interval) = self.time_interval {
          if interval.amount == 0 {
            return Err(ValidationError::invalid_field_value(
              "time_interval.amount".to_string(),
              "Wait amount must be greater than 0".to_string(),
            ));
          }
        }
      }
      WaitMode::SpecificTime => {
        if self.specific_time.is_none() {
          return Err(ValidationError::invalid_field_value(
            "specific_time".to_string(),
            "Specific time configuration is required for specific time mode".to_string(),
          ));
        }
        if let Some(ref specific_time) = self.specific_time {
          if specific_time.date_time <= Utc::now() {
            return Err(ValidationError::invalid_field_value(
              "specific_time.date_time".to_string(),
              "Target time must be in the future".to_string(),
            ));
          }
        }
      }
      WaitMode::Webhook => {
        if self.webhook_config.is_none() {
          return Err(ValidationError::invalid_field_value(
            "webhook_config".to_string(),
            "Webhook configuration is required for webhook mode".to_string(),
          ));
        }
      }
      WaitMode::Form => {
        if self.form_config.is_none() {
          return Err(ValidationError::invalid_field_value(
            "form_config".to_string(),
            "Form configuration is required for form mode".to_string(),
          ));
        }
        if let Some(ref form_config) = self.form_config {
          if form_config.form_title.trim().is_empty() {
            return Err(ValidationError::invalid_field_value(
              "form_config.form_title".to_string(),
              "Form title cannot be empty".to_string(),
            ));
          }
        }
      }
    }

    // 验证时间限制配置
    if let Some(ref time_limit) = self.time_limit {
      if time_limit.enabled {
        match time_limit.limit_type {
          LimitType::AfterTimeInterval => {
            if time_limit.resume_amount.is_none() || time_limit.resume_amount.unwrap() == 0 {
              return Err(ValidationError::invalid_field_value(
                "time_limit.resume_amount".to_string(),
                "Resume amount must be greater than 0".to_string(),
              ));
            }
          }
          LimitType::AtSpecifiedTime => {
            if time_limit.max_date_and_time.is_none() {
              return Err(ValidationError::invalid_field_value(
                "time_limit.max_date_and_time".to_string(),
                "Max date and time is required for at specified time limit".to_string(),
              ));
            }
          }
        }
      }
    }

    Ok(())
  }

  /// 计算等待时间（毫秒）
  pub fn calculate_wait_duration_ms(&self) -> Option<u64> {
    match self.wait_mode {
      WaitMode::TimeInterval => {
        if let Some(ref interval) = self.time_interval {
          let multiplier = match interval.unit {
            TimeUnit::Seconds => 1,
            TimeUnit::Minutes => 60,
            TimeUnit::Hours => 60 * 60,
            TimeUnit::Days => 60 * 60 * 24,
          };
          Some(interval.amount * multiplier * 1000)
        } else {
          None
        }
      }
      WaitMode::SpecificTime => {
        if let Some(ref specific_time) = self.specific_time {
          let now = Utc::now();
          if specific_time.date_time > now {
            let duration = specific_time.date_time - now;
            Some(duration.num_milliseconds() as u64)
          } else {
            None
          }
        } else {
          None
        }
      }
      WaitMode::Webhook | WaitMode::Form => {
        // 对于 Webhook 和 Form 模式，等待时间由外部触发决定
        // 但可以检查是否有时间限制
        if let Some(ref time_limit) = self.time_limit {
          if time_limit.enabled {
            match time_limit.limit_type {
              LimitType::AfterTimeInterval => {
                if let Some(amount) = time_limit.resume_amount {
                  if let Some(unit) = &time_limit.resume_unit {
                    let multiplier = match unit {
                      TimeUnit::Seconds => 1,
                      TimeUnit::Minutes => 60,
                      TimeUnit::Hours => 60 * 60,
                      TimeUnit::Days => 60 * 60 * 24,
                    };
                    Some(amount * multiplier * 1000)
                  } else {
                    None
                  }
                } else {
                  None
                }
              }
              LimitType::AtSpecifiedTime => {
                if let Some(max_time) = time_limit.max_date_and_time {
                  let now = Utc::now();
                  if max_time > now {
                    let duration = max_time - now;
                    Some(duration.num_milliseconds() as u64)
                  } else {
                    None
                  }
                } else {
                  None
                }
              }
            }
          } else {
            None // 无时间限制，无限等待
          }
        } else {
          None // 无时间限制，无限等待
        }
      }
    }
  }

  /// 是否需要短时间等待（小于65秒）
  pub fn is_short_wait(&self) -> bool {
    if let Some(duration_ms) = self.calculate_wait_duration_ms() { duration_ms < 65_000 } else { false }
  }

  /// 检查是否超过时间限制
  pub fn is_time_limit_exceeded(&self) -> bool {
    if let Some(ref time_limit) = self.time_limit {
      if time_limit.enabled {
        match time_limit.limit_type {
          LimitType::AtSpecifiedTime => {
            if let Some(max_time) = time_limit.max_date_and_time {
              return Utc::now() > max_time;
            }
          }
          LimitType::AfterTimeInterval => {
            // 这个检查需要在 WaitTracker 中进行
            // 因为我们需要知道等待开始的时间
          }
        }
      }
    }
    false
  }
}

impl Default for WaitMode {
  fn default() -> Self {
    Self::TimeInterval
  }
}

impl Default for TimeUnit {
  fn default() -> Self {
    Self::Minutes
  }
}

impl Default for HttpMethod {
  fn default() -> Self {
    Self::Post
  }
}

impl Default for ResponseMode {
  fn default() -> Self {
    Self::OnReceived
  }
}

impl Default for AuthenticationType {
  fn default() -> Self {
    Self::None
  }
}

impl Default for LimitType {
  fn default() -> Self {
    Self::AfterTimeInterval
  }
}

impl Default for WaitConfig {
  fn default() -> Self {
    Self {
      wait_mode: WaitMode::TimeInterval,
      time_interval: Some(TimeIntervalConfig { amount: 1, unit: TimeUnit::Minutes }),
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    }
  }
}

pub struct WaitNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl WaitNode {
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(WaitV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinitionBuilder {
    let mut base = NodeDefinitionBuilder::default();
    base
      .kind(NodeKind::from(WAIT_NODE_KIND))
      .groups(vec![NodeGroupKind::Transform, NodeGroupKind::Input])
      .display_name("Wait")
      .description("在工作流执行过程中添加等待机制。支持时间间隔、特定时间、Webhook 触发和表单提交等待。")
      .icon("pause-circle")
      .version(Version::new(1, 0, 0));
    base
  }
}

impl Node for WaitNode {
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

#[cfg(test)]
mod tests {
  use chrono::Utc;
  use hetumind_core::workflow::{ConnectionKind, NodeGroupKind};

  use super::*;

  #[test]
  fn test_node_metadata() {
    let node = WaitNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    assert_eq!(definition.kind.as_ref(), "hetumind_nodes::Wait");
    assert_eq!(&definition.groups, &[NodeGroupKind::Transform, NodeGroupKind::Input]);
    assert_eq!(&definition.display_name, "Wait");
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);
  }

  #[test]
  fn test_node_ports() {
    let node = WaitNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    let input_ports = &definition.inputs[..];
    assert_eq!(input_ports.len(), 1);
    assert_eq!(input_ports[0].kind, ConnectionKind::Main);

    let output_ports = &definition.outputs[..];
    assert_eq!(output_ports.len(), 1);
    assert_eq!(output_ports[0].kind, ConnectionKind::Main);
  }

  #[test]
  fn test_wait_mode_equality() {
    assert_eq!(WaitMode::TimeInterval, WaitMode::TimeInterval);
    assert_ne!(WaitMode::TimeInterval, WaitMode::Webhook);

    // 测试序列化和反序列化
    let wait_mode = WaitMode::SpecificTime;
    let serialized = serde_json::to_string(&wait_mode).unwrap();
    let deserialized: WaitMode = serde_json::from_str(&serialized).unwrap();
    assert_eq!(wait_mode, deserialized);
  }

  #[test]
  fn test_time_unit_equality() {
    assert_eq!(TimeUnit::Minutes, TimeUnit::Minutes);
    assert_ne!(TimeUnit::Minutes, TimeUnit::Hours);

    // 测试序列化和反序列化
    let time_unit = TimeUnit::Seconds;
    let serialized = serde_json::to_string(&time_unit).unwrap();
    let deserialized: TimeUnit = serde_json::from_str(&serialized).unwrap();
    assert_eq!(time_unit, deserialized);
  }

  #[test]
  fn test_time_interval_config_validation() {
    // 有效的配置
    let valid_config = WaitConfig {
      wait_mode: WaitMode::TimeInterval,
      time_interval: Some(TimeIntervalConfig { amount: 5, unit: TimeUnit::Minutes }),
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert!(valid_config.validate().is_ok());

    // 无效的配置（amount 为 0）
    let invalid_config = WaitConfig {
      wait_mode: WaitMode::TimeInterval,
      time_interval: Some(TimeIntervalConfig { amount: 0, unit: TimeUnit::Minutes }),
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert!(invalid_config.validate().is_err());

    // 缺少 time_interval 配置
    let missing_config = WaitConfig {
      wait_mode: WaitMode::TimeInterval,
      time_interval: None,
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert!(missing_config.validate().is_err());
  }

  #[test]
  fn test_specific_time_config_validation() {
    // 有效的配置
    let future_time = Utc::now() + chrono::Duration::hours(1);
    let valid_config = WaitConfig {
      wait_mode: WaitMode::SpecificTime,
      time_interval: None,
      specific_time: Some(SpecificTimeConfig { date_time: future_time, timezone: Some("UTC".to_string()) }),
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert!(valid_config.validate().is_ok());

    // 无效的配置（过去的时间）
    let past_time = Utc::now() - chrono::Duration::hours(1);
    let invalid_config = WaitConfig {
      wait_mode: WaitMode::SpecificTime,
      time_interval: None,
      specific_time: Some(SpecificTimeConfig { date_time: past_time, timezone: None }),
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert!(invalid_config.validate().is_err());
  }

  #[test]
  fn test_webhook_config_validation() {
    // 有效的配置
    let valid_config = WaitConfig {
      wait_mode: WaitMode::Webhook,
      time_interval: None,
      specific_time: None,
      webhook_config: Some(WebhookConfig {
        http_method: HttpMethod::Post,
        response_mode: ResponseMode::OnReceived,
        webhook_suffix: Some("test-webhook".to_string()),
        authentication_type: Some(AuthenticationType::BasicAuth),
        response_data: None,
        response_headers: None,
        response_status_code: Some(200),
      }),
      form_config: None,
      time_limit: None,
    };
    assert!(valid_config.validate().is_ok());

    // 缺少 webhook 配置
    let missing_config = WaitConfig {
      wait_mode: WaitMode::Webhook,
      time_interval: None,
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert!(missing_config.validate().is_err());
  }

  #[test]
  fn test_form_config_validation() {
    // 有效的配置
    let valid_config = WaitConfig {
      wait_mode: WaitMode::Form,
      time_interval: None,
      specific_time: None,
      webhook_config: None,
      form_config: Some(FormConfig {
        form_title: "Test Form".to_string(),
        form_description: Some("Please fill out this form".to_string()),
        form_fields: None,
        redirect_url: None,
        show_submit_button: Some(true),
        show_back_button: Some(false),
        button_labels: None,
      }),
      time_limit: None,
    };
    assert!(valid_config.validate().is_ok());

    // 无效的配置（空的表单标题）
    let invalid_config = WaitConfig {
      wait_mode: WaitMode::Form,
      time_interval: None,
      specific_time: None,
      webhook_config: None,
      form_config: Some(FormConfig {
        form_title: "".to_string(),
        form_description: None,
        form_fields: None,
        redirect_url: None,
        show_submit_button: None,
        show_back_button: None,
        button_labels: None,
      }),
      time_limit: None,
    };
    assert!(invalid_config.validate().is_err());
  }

  #[test]
  fn test_calculate_wait_duration() {
    // 时间间隔
    let interval_config = WaitConfig {
      wait_mode: WaitMode::TimeInterval,
      time_interval: Some(TimeIntervalConfig { amount: 5, unit: TimeUnit::Minutes }),
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert_eq!(interval_config.calculate_wait_duration_ms(), Some(5 * 60 * 1000));

    // 特定时间
    let future_time = Utc::now() + chrono::Duration::hours(2);
    let specific_config = WaitConfig {
      wait_mode: WaitMode::SpecificTime,
      time_interval: None,
      specific_time: Some(SpecificTimeConfig { date_time: future_time, timezone: None }),
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    let duration = specific_config.calculate_wait_duration_ms().unwrap();
    assert!(duration > 0);
    assert!(duration < 3 * 60 * 60 * 1000); // 应该小于3小时
  }

  #[test]
  fn test_is_short_wait() {
    // 短时间等待（30秒）
    let short_config = WaitConfig {
      wait_mode: WaitMode::TimeInterval,
      time_interval: Some(TimeIntervalConfig { amount: 30, unit: TimeUnit::Seconds }),
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert!(short_config.is_short_wait());

    // 长时间等待（2分钟）
    let long_config = WaitConfig {
      wait_mode: WaitMode::TimeInterval,
      time_interval: Some(TimeIntervalConfig { amount: 2, unit: TimeUnit::Minutes }),
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert!(!long_config.is_short_wait());
  }

  #[test]
  fn test_default_config() {
    let default_config = WaitConfig::default();
    assert_eq!(default_config.wait_mode, WaitMode::TimeInterval);
    assert!(default_config.time_interval.is_some());
    assert_eq!(default_config.time_interval.as_ref().unwrap().amount, 1);
    assert_eq!(default_config.time_interval.as_ref().unwrap().unit, TimeUnit::Minutes);
  }
}
