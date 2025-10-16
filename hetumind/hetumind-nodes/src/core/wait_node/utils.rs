//! Wait 节点工具函数
//!
//! 提供等待计算、格式化和状态管理的核心工具函数。

use hetumind_core::workflow::{NodeExecutionError, ValidationError};
use serde_json::{Value, json};
use std::collections::HashMap;

use super::{
  FormConfig, FormField, FormFieldType, LimitType, TimeLimitConfig, TimeUnit, WaitConfig, WaitMode, WebhookConfig,
};

/// 简单的 HTML 转义函数
#[allow(dead_code)]
fn escape_html(text: &str) -> String {
  text
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#x27;")
}

/// HTML 属性转义函数
#[allow(dead_code)]
fn escape_html_attribute(text: &str) -> String {
  text
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#x27;")
    .replace('\n', "&#10;")
    .replace('\r', "&#13;")
    .replace('\t', "&#9;")
}

/// 等待信息结构
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WaitInfo {
  /// 等待模式
  pub mode: WaitMode,
  /// 等待消息
  pub message: String,
  /// 目标时间（如果适用）
  pub target_time: Option<chrono::DateTime<chrono::Utc>>,
  /// 等待持续时间（毫秒）
  pub duration_ms: Option<u64>,
  /// Webhook URL（如果适用）
  pub webhook_url: Option<String>,
  /// 表单 URL（如果适用）
  pub form_url: Option<String>,
  /// 是否已超时
  pub is_timeout: bool,
  /// 创建时间
  pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Webhook 响应数据
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WebhookResponse {
  /// 响应状态码
  pub status_code: u16,
  /// 响应头
  pub headers: HashMap<String, String>,
  /// 响应体
  pub body: Value,
  /// 响应数据
  pub response_data: Option<Value>,
}

/// 计算等待持续时间
///
/// 根据等待配置计算需要等待的时间（毫秒）
///
/// # 参数
/// - `config`: 等待配置
///
/// # 返回
/// 返回等待持续时间（毫秒），如果无法计算则返回 None
pub fn calculate_wait_duration(config: &WaitConfig) -> Option<u64> {
  match config.wait_mode {
    WaitMode::TimeInterval => {
      if let Some(ref interval) = config.time_interval {
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
      if let Some(ref specific_time) = config.specific_time {
        let now = chrono::Utc::now();
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
      if let Some(ref time_limit) = config.time_limit {
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
                let now = chrono::Utc::now();
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

/// 格式化等待消息
///
/// 根据等待配置生成用户友好的等待消息
///
/// # 参数
/// - `config`: 等待配置
///
/// # 返回
/// 返回格式化的等待消息字符串
pub fn format_wait_message(config: &WaitConfig) -> String {
  match config.wait_mode {
    WaitMode::TimeInterval => {
      if let Some(ref interval) = config.time_interval {
        let unit_str = match interval.unit {
          TimeUnit::Seconds => {
            if interval.amount == 1 {
              "秒"
            } else {
              "秒"
            }
          }
          TimeUnit::Minutes => {
            if interval.amount == 1 {
              "分钟"
            } else {
              "分钟"
            }
          }
          TimeUnit::Hours => {
            if interval.amount == 1 {
              "小时"
            } else {
              "小时"
            }
          }
          TimeUnit::Days => {
            if interval.amount == 1 {
              "天"
            } else {
              "天"
            }
          }
        };
        format!("等待 {} {}", interval.amount, unit_str)
      } else {
        "等待指定时间".to_string()
      }
    }
    WaitMode::SpecificTime => {
      if let Some(ref specific_time) = config.specific_time {
        format!("等待到 {}", specific_time.date_time.format("%Y-%m-%d %H:%M:%S UTC"))
      } else {
        "等待到指定时间".to_string()
      }
    }
    WaitMode::Webhook => {
      let mut message = "等待 Webhook 调用".to_string();
      if let Some(ref time_limit) = config.time_limit {
        if time_limit.enabled {
          message += " (有时间限制)";
        } else {
          message += " (无时间限制)";
        }
      }
      message
    }
    WaitMode::Form => {
      if let Some(ref form_config) = config.form_config {
        let mut message = format!("等待表单提交: {}", form_config.form_title);
        if let Some(ref description) = form_config.form_description {
          message += &format!(" - {}", description);
        }
        if let Some(ref time_limit) = config.time_limit {
          if time_limit.enabled {
            message += " (有时间限制)";
          } else {
            message += " (无时间限制)";
          }
        }
        message
      } else {
        "等待表单提交".to_string()
      }
    }
  }
}

/// 创建等待信息
///
/// 根据等待配置创建详细的等待信息对象
///
/// # 参数
/// - `config`: 等待配置
/// - `workflow_id`: 工作流 ID
///
/// # 返回
/// 返回等待信息对象
pub fn create_wait_info(config: &WaitConfig, workflow_id: &str) -> WaitInfo {
  let message = format_wait_message(config);
  let duration_ms = calculate_wait_duration(config);
  let target_time = match config.wait_mode {
    WaitMode::SpecificTime => config.specific_time.as_ref().map(|st| st.date_time),
    WaitMode::TimeInterval => {
      if let Some(duration) = duration_ms {
        Some(chrono::Utc::now() + chrono::Duration::milliseconds(duration as i64))
      } else {
        None
      }
    }
    WaitMode::Webhook | WaitMode::Form => {
      if let Some(ref time_limit) = config.time_limit {
        if time_limit.enabled {
          match time_limit.limit_type {
            LimitType::AfterTimeInterval => {
              if let Some(duration) = duration_ms {
                Some(chrono::Utc::now() + chrono::Duration::milliseconds(duration as i64))
              } else {
                None
              }
            }
            LimitType::AtSpecifiedTime => time_limit.max_date_and_time,
          }
        } else {
          None
        }
      } else {
        None
      }
    }
  };

  let webhook_url = if matches!(config.wait_mode, WaitMode::Webhook) {
    Some(format!("https://example.com/webhook/wait/{}", workflow_id))
  } else {
    None
  };

  let form_url = if matches!(config.wait_mode, WaitMode::Form) {
    Some(format!("https://example.com/form/wait/{}", workflow_id))
  } else {
    None
  };

  WaitInfo {
    mode: config.wait_mode.clone(),
    message,
    target_time,
    duration_ms,
    webhook_url,
    form_url,
    is_timeout: false,
    created_at: chrono::Utc::now(),
  }
}

/// 创建 Webhook 响应
///
/// 根据 Webhook 配置创建响应数据
///
/// # 参数
/// - `config`: Webhook 配置
/// - `request_data`: 请求数据
///
/// # 返回
/// 返回 Webhook 响应对象
pub fn create_webhook_response(
  config: &WebhookConfig,
  request_data: &Value,
) -> Result<WebhookResponse, NodeExecutionError> {
  let mut headers = HashMap::new();
  headers.insert("Content-Type".to_string(), "application/json".to_string());

  // 添加自定义响应头
  if let Some(ref custom_headers) = config.response_headers {
    for (key, value) in custom_headers {
      headers.insert(key.clone(), value.clone());
    }
  }

  // 构建响应体
  let body = if let Some(ref response_data) = config.response_data {
    response_data.clone()
  } else {
    json!({
      "status": "success",
      "message": "Webhook received",
      "timestamp": chrono::Utc::now().to_rfc3339(),
      "request_data": request_data
    })
  };

  let status_code = config.response_status_code.unwrap_or(200);

  Ok(WebhookResponse { status_code, headers, body, response_data: Some(request_data.clone()) })
}

/// 验证表单数据
///
/// 验证提交的表单数据是否符合配置要求
///
/// # 参数
/// - `form_config`: 表单配置
/// - `form_data`: 表单数据
///
/// # 返回
/// 如果验证成功返回 Ok(())，否则返回 ValidationError
pub fn validate_form_data(form_config: &FormConfig, form_data: &Value) -> Result<(), ValidationError> {
  if let Some(fields) = &form_config.form_fields {
    if let Some(form_obj) = form_data.as_object() {
      for field in fields {
        // 检查必填字段
        if field.required.unwrap_or(false) {
          if !form_obj.contains_key(&field.field_id) {
            return Err(ValidationError::invalid_field_value(
              field.field_id.clone(),
              format!("Field '{}' is required", field.field_label),
            ));
          }
        }

        // 验证字段值
        if let Some(value) = form_obj.get(&field.field_id) {
          validate_field_value(field, value)?;
        }
      }
    }
  }

  Ok(())
}

/// 验证字段值
///
/// 验证单个字段值是否符合配置要求
///
/// # 参数
/// - `field`: 字段配置
/// - `value`: 字段值
///
/// # 返回
/// 如果验证成功返回 Ok(())，否则返回 ValidationError
pub fn validate_field_value(field: &FormField, value: &Value) -> Result<(), ValidationError> {
  match field.field_type {
    // Email validation should always be performed regardless of validation config
    FormFieldType::Email => {
      if let Some(email) = value.as_str() {
        // 简单的邮箱验证
        let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").map_err(|_| {
          ValidationError::invalid_field_value(field.field_id.clone(), "Invalid email regex pattern".to_string())
        })?;

        if !email_regex.is_match(email) {
          return Err(ValidationError::invalid_field_value(field.field_id.clone(), "Invalid email format".to_string()));
        }
      }
    }
    _ => {}
  }

  if let Some(ref validation) = field.validation {
    match field.field_type {
      FormFieldType::Text | FormFieldType::Textarea => {
        if let Some(text) = value.as_str() {
          // 验证长度
          if let Some(min_length) = validation.min_length {
            if text.len() < min_length {
              return Err(ValidationError::invalid_field_value(
                field.field_id.clone(),
                format!("Text must be at least {} characters", min_length),
              ));
            }
          }

          if let Some(max_length) = validation.max_length {
            if text.len() > max_length {
              return Err(ValidationError::invalid_field_value(
                field.field_id.clone(),
                format!("Text must be at most {} characters", max_length),
              ));
            }
          }

          // 验证正则表达式
          if let Some(ref pattern) = validation.pattern {
            let regex = regex::Regex::new(pattern).map_err(|_| {
              ValidationError::invalid_field_value(field.field_id.clone(), "Invalid regex pattern".to_string())
            })?;

            if !regex.is_match(text) {
              let error_msg =
                validation.error_message.as_ref().map(|s| s.as_str()).unwrap_or("Field format is invalid");
              return Err(ValidationError::invalid_field_value(field.field_id.clone(), error_msg.to_string()));
            }
          }
        }
      }
      FormFieldType::Email => {
        // Email validation already handled above, but we might have additional validation rules
        if let Some(min_length) = validation.min_length {
          if let Some(email) = value.as_str() {
            if email.len() < min_length {
              return Err(ValidationError::invalid_field_value(
                field.field_id.clone(),
                format!("Email must be at least {} characters", min_length),
              ));
            }
          }
        }
      }
      FormFieldType::Number => {
        if let Some(number) = value.as_f64() {
          // 验证数值范围
          if let Some(min_value) = validation.min_value {
            if number < min_value {
              return Err(ValidationError::invalid_field_value(
                field.field_id.clone(),
                format!("Number must be at least {}", min_value),
              ));
            }
          }

          if let Some(max_value) = validation.max_value {
            if number > max_value {
              return Err(ValidationError::invalid_field_value(
                field.field_id.clone(),
                format!("Number must be at most {}", max_value),
              ));
            }
          }
        }
      }
      FormFieldType::Date => {
        if let Some(date_str) = value.as_str() {
          // 验证日期格式
          chrono::DateTime::parse_from_rfc3339(date_str).map_err(|_| {
            ValidationError::invalid_field_value(
              field.field_id.clone(),
              "Invalid date format, expected RFC3339".to_string(),
            )
          })?;
        }
      }
      FormFieldType::Select | FormFieldType::Radio => {
        // 验证选项值
        if let Some(ref options) = field.options {
          let valid_values: std::collections::HashSet<String> = options.iter().map(|opt| opt.value.clone()).collect();

          let value_str = value.as_str().unwrap_or("");
          if !valid_values.contains(value_str) {
            return Err(ValidationError::invalid_field_value(
              field.field_id.clone(),
              format!("Invalid option value: {}", value_str),
            ));
          }
        }
      }
      FormFieldType::Checkbox => {
        // 复选框应该是布尔值
        if !value.is_boolean() {
          return Err(ValidationError::invalid_field_value(
            field.field_id.clone(),
            "Checkbox field must be boolean".to_string(),
          ));
        }
      }
      FormFieldType::File => {
        // 文件字段应该包含文件信息
        if let Some(file_obj) = value.as_object() {
          if !file_obj.contains_key("filename") || !file_obj.contains_key("content") {
            return Err(ValidationError::invalid_field_value(
              field.field_id.clone(),
              "File field must contain filename and content".to_string(),
            ));
          }
        } else {
          return Err(ValidationError::invalid_field_value(
            field.field_id.clone(),
            "File field must be an object".to_string(),
          ));
        }
      }
      FormFieldType::Password => {
        // 密码字段验证长度
        if let Some(password) = value.as_str() {
          if let Some(min_length) = validation.min_length {
            if password.len() < min_length {
              return Err(ValidationError::invalid_field_value(
                field.field_id.clone(),
                format!("Password must be at least {} characters", min_length),
              ));
            }
          }
        }
      }
      FormFieldType::Time => {
        // 时间字段验证格式
        if let Some(time_str) = value.as_str() {
          // 验证时间格式 (HH:MM:SS)
          let time_regex = regex::Regex::new(r"^\d{2}:\d{2}:\d{2}$").map_err(|_| {
            ValidationError::invalid_field_value(field.field_id.clone(), "Invalid time regex pattern".to_string())
          })?;

          if !time_regex.is_match(time_str) {
            let error_msg = validation
              .error_message
              .as_ref()
              .map(|s| s.as_str())
              .unwrap_or("Invalid time format, expected HH:MM:SS");
            return Err(ValidationError::invalid_field_value(field.field_id.clone(), error_msg.to_string()));
          }
        }
      }
      FormFieldType::Hidden => {
        // 隐藏字段通常不需要验证
      }
    }
  }

  Ok(())
}

/// 解析时间字符串
///
/// 解析多种格式的时间字符串为 DateTime<Utc>
///
/// # 参数
/// - `time_str`: 时间字符串
///
/// # 返回
/// 返回解析后的 DateTime<Utc>
pub fn parse_time_string(time_str: &str) -> Result<chrono::DateTime<chrono::Utc>, ValidationError> {
  // 尝试 RFC3339 格式
  if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time_str) {
    return Ok(dt.with_timezone(&chrono::Utc));
  }

  // 尝试其他常见格式
  let formats = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M", "%Y/%m/%d %H:%M:%S", "%Y/%m/%d %H:%M", "%Y/%m/%d"];

  // 先尝试日期时间格式
  for format in &formats {
    if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(time_str, format) {
      return Ok(chrono::DateTime::from_naive_utc_and_offset(naive_dt, chrono::Utc));
    }
  }

  // 再尝试纯日期格式
  let date_formats = ["%Y-%m-%d", "%Y/%m/%d"];
  for format in &date_formats {
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(time_str, format) {
      let naive_dt = naive_date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| ValidationError::invalid_field_value("time".to_string(), "Invalid date format".to_string()))?;
      return Ok(chrono::DateTime::from_naive_utc_and_offset(naive_dt, chrono::Utc));
    }
  }

  Err(ValidationError::invalid_field_value("time".to_string(), format!("Unable to parse time string: {}", time_str)))
}

/// 检查是否超过时间限制
///
/// 检查等待是否超过了配置的时间限制
///
/// # 参数
/// - `time_limit`: 时间限制配置
/// - `start_time`: 等待开始时间
///
/// # 返回
/// 如果超过时间限制返回 true，否则返回 false
pub fn is_time_limit_exceeded(time_limit: &TimeLimitConfig, start_time: chrono::DateTime<chrono::Utc>) -> bool {
  if !time_limit.enabled {
    return false;
  }

  let now = chrono::Utc::now();

  match time_limit.limit_type {
    LimitType::AfterTimeInterval => {
      if let (Some(amount), Some(unit)) = (time_limit.resume_amount, time_limit.resume_unit.as_ref()) {
        let multiplier = match unit {
          TimeUnit::Seconds => 1,
          TimeUnit::Minutes => 60,
          TimeUnit::Hours => 60 * 60,
          TimeUnit::Days => 60 * 60 * 24,
        };
        let limit_duration = chrono::Duration::seconds((amount * multiplier) as i64);
        start_time + limit_duration < now
      } else {
        false
      }
    }
    LimitType::AtSpecifiedTime => {
      if let Some(max_time) = time_limit.max_date_and_time {
        max_time < now
      } else {
        false
      }
    }
  }
}

/// 生成唯一的 Webhook ID
///
/// 生成用于 Webhook 识别的唯一 ID
///
/// # 参数
/// - `workflow_id`: 工作流 ID
/// - `node_id`: 节点 ID
/// - `execution_id`: 执行 ID
///
/// # 返回
/// 返回唯一的 Webhook ID
pub fn generate_webhook_id(workflow_id: &str, node_id: &str, execution_id: &str) -> String {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};

  let mut hasher = DefaultHasher::new();
  workflow_id.hash(&mut hasher);
  node_id.hash(&mut hasher);
  execution_id.hash(&mut hasher);

  format!("webhook_{:x}", hasher.finish())
}

/// 创建表单 HTML
///
/// 根据 FormConfig 生成表单的 HTML
///
/// # 参数
/// - `form_config`: 表单配置
/// - `form_url`: 表单提交 URL
///
/// # 返回
/// 返回表单 HTML 字符串
pub fn create_form_html(form_config: &FormConfig, form_url: &str) -> String {
  let mut html = String::new();

  html.push_str("<!DOCTYPE html>\n");
  html.push_str("<html>\n");
  html.push_str("<head>\n");
  html.push_str("    <meta charset=\"UTF-8\">\n");
  html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
  html.push_str(&format!("    <title>{}</title>\n", escape_html(&form_config.form_title)));
  html.push_str("    <style>\n");
  html.push_str("        body { font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px; }\n");
  html.push_str("        .form-group { margin-bottom: 15px; }\n");
  html.push_str("        label { display: block; margin-bottom: 5px; font-weight: bold; }\n");
  html.push_str(
    "        input, textarea, select { width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 4px; }\n",
  );
  html.push_str("        .buttons { margin-top: 20px; }\n");
  html.push_str(
    "        button { padding: 10px 20px; margin-right: 10px; border: none; border-radius: 4px; cursor: pointer; }\n",
  );
  html.push_str("        .submit-btn { background-color: #007bff; color: white; }\n");
  html.push_str("        .back-btn { background-color: #6c757d; color: white; }\n");
  html.push_str("        .required { color: red; }\n");
  html.push_str("    </style>\n");
  html.push_str("</head>\n");
  html.push_str("<body>\n");

  html.push_str("    <h1>");
  html.push_str(&escape_html(&form_config.form_title));
  html.push_str("</h1>\n");

  if let Some(ref description) = form_config.form_description {
    html.push_str(&format!("    <p>{}</p>\n", escape_html(description)));
  }

  html.push_str("    <form method=\"POST\" action=\"");
  html.push_str(&escape_html_attribute(form_url));
  html.push_str("\">\n");

  if let Some(ref fields) = form_config.form_fields {
    for field in fields {
      html.push_str("        <div class=\"form-group\">\n");
      html.push_str(&format!("            <label for=\"{}\">", escape_html_attribute(&field.field_id)));
      html.push_str(&escape_html(&field.field_label));
      if field.required.unwrap_or(false) {
        html.push_str(" <span class=\"required\">*</span>");
      }
      html.push_str(":</label>\n");

      match field.field_type {
        FormFieldType::Text => {
          html.push_str(&format!(
            "            <input type=\"text\" id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.required.unwrap_or(false) {
            html.push_str(" required");
          }
          if let Some(ref default) = field.default_value {
            if let Some(default_str) = default.as_str() {
              html.push_str(&format!(" value=\"{}\"", escape_html_attribute(default_str)));
            }
          }
          html.push_str(">\n");
        }
        FormFieldType::Textarea => {
          html.push_str(&format!(
            "            <textarea id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.required.unwrap_or(false) {
            html.push_str(" required");
          }
          html.push_str(">\n");
          if let Some(ref default) = field.default_value {
            if let Some(default_str) = default.as_str() {
              html.push_str(&escape_html(default_str));
            }
          }
          html.push_str("            </textarea>\n");
        }
        FormFieldType::Number => {
          html.push_str(&format!(
            "            <input type=\"number\" id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.required.unwrap_or(false) {
            html.push_str(" required");
          }
          if let Some(ref default) = field.default_value {
            if let Some(default_num) = default.as_f64() {
              html.push_str(&format!(" value=\"{}\"", default_num));
            }
          }
          html.push_str(">\n");
        }
        FormFieldType::Email => {
          html.push_str(&format!(
            "            <input type=\"email\" id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.required.unwrap_or(false) {
            html.push_str(" required");
          }
          html.push_str(">\n");
        }
        FormFieldType::Select => {
          html.push_str(&format!(
            "            <select id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.required.unwrap_or(false) {
            html.push_str(" required");
          }
          html.push_str(">\n");

          if let Some(ref options) = field.options {
            for option in options {
              html.push_str("                <option value=\"");
              html.push_str(&escape_html_attribute(&option.value));
              if option.selected.unwrap_or(false) {
                html.push_str("\" selected");
              }
              html.push_str("\">");
              html.push_str(&escape_html(&option.label));
              html.push_str("</option>\n");
            }
          }

          html.push_str("            </select>\n");
        }
        FormFieldType::Radio => {
          if let Some(ref options) = field.options {
            for option in options {
              html.push_str("            <div>\n");
              html.push_str("                <input type=\"radio\" id=\"");
              html.push_str(&escape_html_attribute(&format!("{}_{}", field.field_id, option.value)));
              html.push_str("\" name=\"");
              html.push_str(&escape_html_attribute(&field.field_id));
              html.push_str("\" value=\"");
              html.push_str(&escape_html_attribute(&option.value));
              if option.selected.unwrap_or(false) {
                html.push_str("\" checked");
              }
              html.push_str("\">\n");
              html.push_str("                <label for=\"");
              html.push_str(&escape_html_attribute(&format!("{}_{}", field.field_id, option.value)));
              html.push_str("\">");
              html.push_str(&escape_html(&option.label));
              html.push_str("</label>\n");
              html.push_str("            </div>\n");
            }
          }
        }
        FormFieldType::Checkbox => {
          html.push_str(&format!(
            "            <input type=\"checkbox\" id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.default_value.as_ref().and_then(|v| v.as_bool()).unwrap_or(false) {
            html.push_str(" checked");
          }
          html.push_str(">\n");
        }
        FormFieldType::Date => {
          html.push_str(&format!(
            "            <input type=\"date\" id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.required.unwrap_or(false) {
            html.push_str(" required");
          }
          html.push_str(">\n");
        }
        FormFieldType::File => {
          html.push_str(&format!(
            "            <input type=\"file\" id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.required.unwrap_or(false) {
            html.push_str(" required");
          }
          html.push_str(">\n");
        }
        FormFieldType::Password => {
          html.push_str(&format!(
            "            <input type=\"password\" id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.required.unwrap_or(false) {
            html.push_str(" required");
          }
          html.push_str(">\n");
        }
        FormFieldType::Time => {
          html.push_str(&format!(
            "            <input type=\"time\" id=\"{}\" name=\"{}\"",
            escape_html_attribute(&field.field_id),
            escape_html_attribute(&field.field_id)
          ));
          if field.required.unwrap_or(false) {
            html.push_str(" required");
          }
          html.push_str(">\n");
        }
        FormFieldType::Hidden => {
          if let Some(ref default) = field.default_value {
            html.push_str(&format!(
              "            <input type=\"hidden\" id=\"{}\" name=\"{}\" value=\"{}\"",
              escape_html_attribute(&field.field_id),
              escape_html_attribute(&field.field_id),
              escape_html_attribute(&default.to_string())
            ));
            html.push_str(">\n");
          }
        }
      }

      html.push_str("        </div>\n");
    }
  }

  html.push_str("        <div class=\"buttons\">\n");

  if form_config.show_submit_button.unwrap_or(true) {
    let submit_text = form_config
      .button_labels
      .as_ref()
      .and_then(|labels| labels.submit.as_ref())
      .cloned()
      .unwrap_or_else(|| "提交".to_string());
    html.push_str(&format!(
      "            <button type=\"submit\" class=\"submit-btn\">{}</button>\n",
      escape_html(&submit_text)
    ));
  }

  if form_config.show_back_button.unwrap_or(false) {
    let back_text = form_config
      .button_labels
      .as_ref()
      .and_then(|labels| labels.back.as_ref())
      .cloned()
      .unwrap_or_else(|| "返回".to_string());
    html.push_str(&format!(
      "            <button type=\"button\" class=\"back-btn\" onclick=\"history.back()\">{}</button>\n",
      escape_html(&back_text)
    ));
  }

  html.push_str("        </div>\n");
  html.push_str("    </form>\n");
  html.push_str("</body>\n");
  html.push_str("</html>\n");

  html
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::wait_node::{
    AuthenticationType, FieldValidation, HttpMethod, ResponseMode, SpecificTimeConfig, TimeIntervalConfig,
  };
  use chrono::Utc;
  use chrono::{Datelike, Timelike};

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
    assert_eq!(calculate_wait_duration(&interval_config), Some(5 * 60 * 1000));

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
    let duration = calculate_wait_duration(&specific_config).unwrap();
    assert!(duration > 0);
    assert!(duration < 3 * 60 * 60 * 1000); // 应该小于3小时
  }

  #[test]
  fn test_format_wait_message() {
    let interval_config = WaitConfig {
      wait_mode: WaitMode::TimeInterval,
      time_interval: Some(TimeIntervalConfig { amount: 5, unit: TimeUnit::Minutes }),
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };
    assert_eq!(format_wait_message(&interval_config), "等待 5 分钟");

    let webhook_config = WaitConfig {
      wait_mode: WaitMode::Webhook,
      time_interval: None,
      specific_time: None,
      webhook_config: Some(WebhookConfig {
        http_method: HttpMethod::Post,
        response_mode: ResponseMode::OnReceived,
        webhook_suffix: None,
        authentication_type: None,
        response_data: None,
        response_headers: None,
        response_status_code: Some(200),
      }),
      form_config: None,
      time_limit: Some(TimeLimitConfig {
        enabled: true,
        limit_type: LimitType::AfterTimeInterval,
        resume_amount: Some(60),
        resume_unit: Some(TimeUnit::Minutes),
        max_date_and_time: None,
      }),
    };
    assert_eq!(format_wait_message(&webhook_config), "等待 Webhook 调用 (有时间限制)");
  }

  #[test]
  fn test_create_wait_info() {
    let config = WaitConfig {
      wait_mode: WaitMode::TimeInterval,
      time_interval: Some(TimeIntervalConfig { amount: 30, unit: TimeUnit::Seconds }),
      specific_time: None,
      webhook_config: None,
      form_config: None,
      time_limit: None,
    };

    let wait_info = create_wait_info(&config, "workflow-123");
    assert_eq!(wait_info.mode, WaitMode::TimeInterval);
    assert_eq!(wait_info.message, "等待 30 秒");
    assert_eq!(wait_info.duration_ms, Some(30_000));
    assert!(wait_info.webhook_url.is_none());
    assert!(wait_info.form_url.is_none());
    assert!(!wait_info.is_timeout);
  }

  #[test]
  fn test_create_webhook_response() {
    let config = WebhookConfig {
      http_method: HttpMethod::Post,
      response_mode: ResponseMode::OnReceived,
      webhook_suffix: Some("test".to_string()),
      authentication_type: Some(AuthenticationType::BasicAuth),
      response_data: Some(json!({"custom": "response"})),
      response_headers: Some({
        let mut headers = std::collections::HashMap::new();
        headers.insert("X-Custom".to_string(), "value".to_string());
        headers
      }),
      response_status_code: Some(201),
    };

    let request_data = json!({"input": "data"});
    let response = create_webhook_response(&config, &request_data).unwrap();

    assert_eq!(response.status_code, 201);
    assert_eq!(response.headers.get("X-Custom"), Some(&"value".to_string()));
    assert_eq!(response.response_data, Some(json!({"input": "data"})));
    assert_eq!(response.body, json!({"custom": "response"}));
  }

  #[test]
  fn test_validate_form_data() {
    let form_config = FormConfig {
      form_title: "Test Form".to_string(),
      form_description: None,
      form_fields: Some(vec![
        FormField {
          field_id: "email".to_string(),
          field_label: "Email".to_string(),
          field_type: FormFieldType::Email,
          required: Some(true),
          default_value: None,
          options: None,
          validation: None,
        },
        FormField {
          field_id: "name".to_string(),
          field_label: "Name".to_string(),
          field_type: FormFieldType::Text,
          required: Some(true),
          default_value: None,
          options: None,
          validation: Some(FieldValidation {
            min_length: Some(2),
            max_length: Some(50),
            min_value: None,
            max_value: None,
            pattern: None,
            error_message: None,
          }),
        },
      ]),
      redirect_url: None,
      show_submit_button: None,
      show_back_button: None,
      button_labels: None,
    };

    // 有效的表单数据
    let valid_data = json!({
      "email": "test@example.com",
      "name": "John Doe"
    });
    assert!(validate_form_data(&form_config, &valid_data).is_ok());

    // 缺少必填字段
    let missing_field = json!({
      "email": "test@example.com"
    });
    assert!(validate_form_data(&form_config, &missing_field).is_err());

    // 无效的邮箱格式
    let invalid_email = json!({
      "email": "invalid-email",
      "name": "John Doe"
    });
    assert!(validate_form_data(&form_config, &invalid_email).is_err());

    // 名字太短
    let short_name = json!({
      "email": "test@example.com",
      "name": "A"
    });
    assert!(validate_form_data(&form_config, &short_name).is_err());
  }

  #[test]
  fn test_parse_time_string() {
    // RFC3339 格式
    let rfc_time = "2024-12-31T23:59:59Z";
    let parsed = parse_time_string(rfc_time).unwrap();
    assert_eq!(parsed.year(), 2024);
    assert_eq!(parsed.month(), 12);
    assert_eq!(parsed.day(), 31);

    // 标准日期时间格式
    let standard_time = "2024-12-31 23:59:59";
    let parsed = parse_time_string(standard_time).unwrap();
    assert_eq!(parsed.year(), 2024);
    assert_eq!(parsed.month(), 12);
    assert_eq!(parsed.day(), 31);

    // 仅日期格式
    let date_only = "2024-12-31";
    let parsed = parse_time_string(date_only).unwrap();
    assert_eq!(parsed.year(), 2024);
    assert_eq!(parsed.month(), 12);
    assert_eq!(parsed.day(), 31);
    assert_eq!(parsed.hour(), 0);
    assert_eq!(parsed.minute(), 0);

    // 无效格式
    let invalid_time = "invalid time";
    assert!(parse_time_string(invalid_time).is_err());
  }

  #[test]
  fn test_is_time_limit_exceeded() {
    let start_time = Utc::now() - chrono::Duration::minutes(30);

    // 时间间隔限制
    let interval_limit = TimeLimitConfig {
      enabled: true,
      limit_type: LimitType::AfterTimeInterval,
      resume_amount: Some(15), // 15分钟
      resume_unit: Some(TimeUnit::Minutes),
      max_date_and_time: None,
    };
    assert!(is_time_limit_exceeded(&interval_limit, start_time));

    // 特定时间限制
    let max_time = Utc::now() - chrono::Duration::minutes(5);
    let specific_limit = TimeLimitConfig {
      enabled: true,
      limit_type: LimitType::AtSpecifiedTime,
      resume_amount: None,
      resume_unit: None,
      max_date_and_time: Some(max_time),
    };
    assert!(is_time_limit_exceeded(&specific_limit, start_time));

    // 未启用限制
    let disabled_limit = TimeLimitConfig {
      enabled: false,
      limit_type: LimitType::AfterTimeInterval,
      resume_amount: None,
      resume_unit: None,
      max_date_and_time: None,
    };
    assert!(!is_time_limit_exceeded(&disabled_limit, start_time));
  }

  #[test]
  fn test_generate_webhook_id() {
    let webhook_id = generate_webhook_id("workflow-123", "node-456", "execution-789");
    assert!(webhook_id.starts_with("webhook_"));
    assert!(webhook_id.len() > 10); // 应该有足够的长度
  }

  #[test]
  fn test_create_form_html() {
    let form_config = FormConfig {
      form_title: "Test Form".to_string(),
      form_description: Some("Please fill out this form".to_string()),
      form_fields: Some(vec![
        FormField {
          field_id: "name".to_string(),
          field_label: "Name".to_string(),
          field_type: FormFieldType::Text,
          required: Some(true),
          default_value: None,
          options: None,
          validation: None,
        },
        FormField {
          field_id: "email".to_string(),
          field_label: "Email".to_string(),
          field_type: FormFieldType::Email,
          required: Some(true),
          default_value: None,
          options: None,
          validation: None,
        },
      ]),
      redirect_url: None,
      show_submit_button: Some(true),
      show_back_button: Some(false),
      button_labels: None,
    };

    let form_url = "https://example.com/submit";
    let html = create_form_html(&form_config, form_url);

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<title>Test Form</title>"));
    assert!(html.contains("Please fill out this form"));
    assert!(html.contains("input type=\"text\""));
    assert!(html.contains("input type=\"email\""));
    assert!(html.contains("required"));
    assert!(html.contains("submit-btn"));
    // Check that there's no actual back button element (only CSS class should be present)
    assert!(!html.contains("<button type=\"button\" class=\"back-btn\""));
  }
}
