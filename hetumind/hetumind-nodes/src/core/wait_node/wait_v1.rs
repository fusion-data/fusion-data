use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, ExecutionData, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition,
    NodeDefinitionBuilder, NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind,
    OutputPortConfig, RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{
  AuthenticationType, FormConfig, HttpMethod, LimitType, ResponseMode, TimeLimitConfig, TimeUnit, WaitConfig, WaitMode,
  WebhookConfig, utils::format_wait_message,
};

/// Wait 等待节点 V1
///
/// 用于在工作流执行过程中添加等待机制，支持四种等待模式：
///
/// # 等待模式
/// - `timeInterval`: 等待指定的时间长度
/// - `specificTime`: 等待到指定的日期和时间
/// - `webhook`: 等待 Webhook 调用来恢复执行
/// - `form`: 等待用户提交表单来恢复执行
///
/// # 优化策略
/// - 小于 65 秒的短时间等待使用内存中的 Promise
/// - 大于等于 65 秒的长时间等待使用数据库存储
/// - 支持时间限制配置，避免无限等待
///
/// # 输入
/// - 接收任意 JSON 数据，等待完成后原样传递
///
/// # 输出
/// - Webhook 和 Form 模式：输出包含触发数据的 JSON
/// - 时间等待模式：原样输出输入数据
#[derive(Debug)]
pub struct WaitV1 {
  pub definition: Arc<NodeDefinition>,
}

#[async_trait]
impl NodeExecutable for WaitV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "开始执行 Wait 等待节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("Wait 节点接收到 {} 个输入项", input_data.len());
      input_data
    } else {
      log::warn!("Wait 节点没有接收到输入数据，返回空结果");
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_items(Default::default())],
      )]));
    };

    // 获取等待模式
    let wait_mode: WaitMode = node.get_parameter("wait_mode")?;

    // 构建等待配置
    let config = match wait_mode {
      WaitMode::TimeInterval => {
        let amount: u64 = node.get_parameter("time_amount")?;
        let unit: TimeUnit = node.get_parameter("time_unit")?;

        WaitConfig {
          wait_mode: WaitMode::TimeInterval,
          time_interval: Some(super::TimeIntervalConfig { amount, unit }),
          specific_time: None,
          webhook_config: None,
          form_config: None,
          time_limit: None,
        }
      }
      WaitMode::SpecificTime => {
        let date_time_str: String = node.get_parameter("specific_date_time")?;
        let date_time = chrono::DateTime::parse_from_rfc3339(&date_time_str)
          .map_err(|e| NodeExecutionError::ConfigurationError(format!("Invalid date time format: {}", e)))?
          .with_timezone(&chrono::Utc);

        WaitConfig {
          wait_mode: WaitMode::SpecificTime,
          time_interval: None,
          specific_time: Some(super::SpecificTimeConfig {
            date_time,
            timezone: None, // TODO: 从节点参数获取时区
          }),
          webhook_config: None,
          form_config: None,
          time_limit: None,
        }
      }
      WaitMode::Webhook => {
        let http_method: HttpMethod = node.get_parameter("webhook_http_method")?;
        let response_mode: ResponseMode = node.get_parameter("webhook_response_mode")?;
        let webhook_suffix: Option<String> = node.get_optional_parameter("webhook_suffix");
        let authentication_type: Option<AuthenticationType> =
          node.get_optional_parameter("webhook_authentication_type");

        // 处理时间限制
        let time_limit = if let Some(limit_enabled) = node.get_optional_parameter::<bool>("webhook_limit_wait_time") {
          if limit_enabled {
            let limit_type: LimitType = node.get_parameter("webhook_limit_type")?;
            match limit_type {
              LimitType::AfterTimeInterval => {
                let resume_amount: u64 = node.get_parameter("webhook_resume_amount")?;
                let resume_unit: TimeUnit = node.get_parameter("webhook_resume_unit")?;
                Some(TimeLimitConfig {
                  enabled: true,
                  limit_type: LimitType::AfterTimeInterval,
                  resume_amount: Some(resume_amount),
                  resume_unit: Some(resume_unit),
                  max_date_and_time: None,
                })
              }
              LimitType::AtSpecifiedTime => {
                let max_date_time_str: String = node.get_parameter("webhook_max_date_time")?;
                let max_date_time = chrono::DateTime::parse_from_rfc3339(&max_date_time_str)
                  .map_err(|e| NodeExecutionError::ConfigurationError(format!("Invalid max date time format: {}", e)))?
                  .with_timezone(&chrono::Utc);
                Some(TimeLimitConfig {
                  enabled: true,
                  limit_type: LimitType::AtSpecifiedTime,
                  resume_amount: None,
                  resume_unit: None,
                  max_date_and_time: Some(max_date_time),
                })
              }
            }
          } else {
            None
          }
        } else {
          None
        };

        WaitConfig {
          wait_mode: WaitMode::Webhook,
          time_interval: None,
          specific_time: None,
          webhook_config: Some(WebhookConfig {
            http_method,
            response_mode,
            webhook_suffix,
            authentication_type,
            response_data: None,
            response_headers: None,
            response_status_code: Some(200),
          }),
          form_config: None,
          time_limit,
        }
      }
      WaitMode::Form => {
        let form_title: String = node.get_parameter("form_title")?;
        let form_description: Option<String> = node.get_optional_parameter("form_description");
        let redirect_url: Option<String> = node.get_optional_parameter("form_redirect_url");
        let show_submit_button: Option<bool> = node.get_optional_parameter("form_show_submit_button");
        let show_back_button: Option<bool> = node.get_optional_parameter("form_show_back_button");

        // 处理时间限制
        let time_limit = if let Some(limit_enabled) = node.get_optional_parameter::<bool>("form_limit_wait_time") {
          if limit_enabled {
            let limit_type: LimitType = node.get_parameter("form_limit_type")?;
            match limit_type {
              LimitType::AfterTimeInterval => {
                let resume_amount: u64 = node.get_parameter("form_resume_amount")?;
                let resume_unit: TimeUnit = node.get_parameter("form_resume_unit")?;
                Some(TimeLimitConfig {
                  enabled: true,
                  limit_type: LimitType::AfterTimeInterval,
                  resume_amount: Some(resume_amount),
                  resume_unit: Some(resume_unit),
                  max_date_and_time: None,
                })
              }
              LimitType::AtSpecifiedTime => {
                let max_date_time_str: String = node.get_parameter("form_max_date_time")?;
                let max_date_time = chrono::DateTime::parse_from_rfc3339(&max_date_time_str)
                  .map_err(|e| NodeExecutionError::ConfigurationError(format!("Invalid max date time format: {}", e)))?
                  .with_timezone(&chrono::Utc);
                Some(TimeLimitConfig {
                  enabled: true,
                  limit_type: LimitType::AtSpecifiedTime,
                  resume_amount: None,
                  resume_unit: None,
                  max_date_and_time: Some(max_date_time),
                })
              }
            }
          } else {
            None
          }
        } else {
          None
        };

        WaitConfig {
          wait_mode: WaitMode::Form,
          time_interval: None,
          specific_time: None,
          webhook_config: None,
          form_config: Some(FormConfig {
            form_title,
            form_description,
            form_fields: None, // TODO: 从节点参数获取表单字段
            redirect_url,
            show_submit_button,
            show_back_button,
            button_labels: None, // TODO: 从节点参数获取按钮标签
          }),
          time_limit,
        }
      }
    };

    // 验证配置
    if let Err(e) = config.validate() {
      log::error!("Wait 配置验证失败: {}", e);
      return Err(NodeExecutionError::ConfigurationError(format!("Invalid Wait configuration: {}", e)));
    }

    log::debug!("等待配置: 模式={:?}, 计算等待时间={:?}ms", config.wait_mode, config.calculate_wait_duration_ms());

    let wait_message = format_wait_message(&config);
    log::info!("Wait 节点: {}", wait_message);

    // 根据等待模式和时间长度选择执行策略
    match config.wait_mode {
      WaitMode::TimeInterval | WaitMode::SpecificTime => {
        // 时间等待模式
        if config.is_short_wait() {
          // 短时间等待：使用 Promise setTimeout
          if let Some(duration_ms) = config.calculate_wait_duration_ms() {
            log::info!("使用短时间等待: {}ms", duration_ms);
            tokio::time::sleep(Duration::from_millis(duration_ms)).await;

            // 直接返回输入数据
            let execution_data: Vec<ExecutionData> =
              input_items.into_iter().map(|item| ExecutionData::new_json(item.json().clone(), None)).collect();

            Ok(make_execution_data_map(vec![(
              ConnectionKind::Main,
              vec![ExecutionDataItems::new_items(execution_data)],
            )]))
          } else {
            // 计算等待时间失败，返回输入数据
            let execution_data: Vec<ExecutionData> =
              input_items.into_iter().map(|item| ExecutionData::new_json(item.json().clone(), None)).collect();

            Ok(make_execution_data_map(vec![(
              ConnectionKind::Main,
              vec![ExecutionDataItems::new_items(execution_data)],
            )]))
          }
        } else {
          // 长时间等待：需要数据库支持
          // TODO: 实现 putExecutionToWait 功能
          log::warn!("长时间等待需要数据库支持，当前直接返回输入数据");

          let execution_data: Vec<ExecutionData> =
            input_items.into_iter().map(|item| ExecutionData::new_json(item.json().clone(), None)).collect();

          Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(execution_data)])]))
        }
      }
      WaitMode::Webhook => {
        // Webhook 等待模式
        log::info!("配置 Webhook 等待模式");

        // TODO: 实现 Webhook 配置和等待功能
        // 当前返回带有等待信息的输入数据
        let mut output_data = input_items.clone();

        // 添加等待信息到输出数据
        for item in &mut output_data {
          let json_value = item.json().clone();
          if let Some(mut json_obj) = json_value.as_object().cloned() {
            json_obj.insert(
              "_wait_info".to_string(),
              json!({
                "mode": "webhook",
                "message": wait_message,
                "webhook_url": format!("https://example.com/webhook/wait/{}", context.workflow.id),
                "created_at": chrono::Utc::now().to_rfc3339()
              }),
            );
            *item = ExecutionData::new_json(json!(json_obj), None);
          }
        }

        let execution_data: Vec<ExecutionData> = output_data;

        Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(execution_data)])]))
      }
      WaitMode::Form => {
        // 表单等待模式
        log::info!("配置表单等待模式");

        // TODO: 实现表单配置和等待功能
        // 当前返回带有等待信息的输入数据
        let mut output_data = input_items.clone();

        // 添加等待信息到输出数据
        for item in &mut output_data {
          let json_value = item.json().clone();
          if let Some(mut json_obj) = json_value.as_object().cloned() {
            json_obj.insert("_wait_info".to_string(), json!({
              "mode": "form",
              "message": wait_message,
              "form_url": format!("https://example.com/form/wait/{}", context.workflow.id),
              "form_title": config.form_config.as_ref().map(|f| &f.form_title).map_or("Please complete the form", |v| v),
              "created_at": chrono::Utc::now().to_rfc3339()
            }));
            *item = ExecutionData::new_json(json!(json_obj), None);
          }
        }

        let execution_data: Vec<ExecutionData> = output_data;

        Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(execution_data)])]))
      }
    }
  }
}

impl TryFrom<NodeDefinitionBuilder> for WaitV1 {
  type Error = RegistrationError;

  fn try_from(mut base: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    base
      .version(Version::new(1, 0, 0))
      .inputs([InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build()])
      .outputs([OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build()])
      .properties([
        // 等待模式配置
        NodeProperty::builder()
          .display_name("等待模式")
          .name("wait_mode")
          .kind(NodePropertyKind::Options)
          .required(true)
          .description("选择等待的类型")
          .value(json!(WaitMode::TimeInterval))
          .options(vec![
            Box::new(NodeProperty::new_option(
              "时间间隔",
              "time_interval",
              json!(WaitMode::TimeInterval),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "特定时间",
              "specific_time",
              json!(WaitMode::SpecificTime),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "Webhook 调用",
              "webhook",
              json!(WaitMode::Webhook),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option("表单提交", "form", json!(WaitMode::Form), NodePropertyKind::String)),
          ])
          .build(),
        // 时间间隔配置
        NodeProperty::builder()
          .display_name("等待时间")
          .name("time_amount")
          .kind(NodePropertyKind::Number)
          .required(false)
          .description("等待的时间数量")
          .placeholder("输入等待时间...")
          .build(),
        NodeProperty::builder()
          .display_name("时间单位")
          .name("time_unit")
          .kind(NodePropertyKind::Options)
          .required(false)
          .description("选择时间单位")
          .value(json!(TimeUnit::Minutes))
          .options(vec![
            Box::new(NodeProperty::new_option("秒", "seconds", json!(TimeUnit::Seconds), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("分钟", "minutes", json!(TimeUnit::Minutes), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("小时", "hours", json!(TimeUnit::Hours), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("天", "days", json!(TimeUnit::Days), NodePropertyKind::String)),
          ])
          .build(),
        // 特定时间配置
        NodeProperty::builder()
          .display_name("特定时间")
          .name("specific_date_time")
          .kind(NodePropertyKind::String)
          .required(false)
          .description("等待到指定的日期和时间 (RFC3339 格式)")
          .placeholder("2024-12-31T23:59:59Z")
          .build(),
        // Webhook 配置
        NodeProperty::builder()
          .display_name("Webhook HTTP 方法")
          .name("webhook_http_method")
          .kind(NodePropertyKind::Options)
          .required(false)
          .description("Webhook 请求的 HTTP 方法")
          .value(json!(HttpMethod::Post))
          .options(vec![
            Box::new(NodeProperty::new_option("GET", "get", json!(HttpMethod::Get), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("POST", "post", json!(HttpMethod::Post), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("PUT", "put", json!(HttpMethod::Put), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("PATCH", "patch", json!(HttpMethod::Patch), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("DELETE", "delete", json!(HttpMethod::Delete), NodePropertyKind::String)),
          ])
          .build(),
        NodeProperty::builder()
          .display_name("Webhook 响应模式")
          .name("webhook_response_mode")
          .kind(NodePropertyKind::Options)
          .required(false)
          .description("Webhook 响应的模式")
          .value(json!(ResponseMode::OnReceived))
          .options(vec![
            Box::new(NodeProperty::new_option(
              "接收到响应时立即响应",
              "on_received",
              json!(ResponseMode::OnReceived),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "最后节点完成后响应",
              "on_execution_finished",
              json!(ResponseMode::OnExecutionFinished),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option("不响应", "never", json!(ResponseMode::Never), NodePropertyKind::String)),
          ])
          .build(),
        NodeProperty::builder()
          .display_name("Webhook 后缀")
          .name("webhook_suffix")
          .kind(NodePropertyKind::String)
          .required(false)
          .description("Webhook URL 的后缀")
          .placeholder("webhook-suffix")
          .build(),
        // 表单配置
        NodeProperty::builder()
          .display_name("表单标题")
          .name("form_title")
          .kind(NodePropertyKind::String)
          .required(false)
          .description("表单的标题")
          .placeholder("请填写表单")
          .build(),
        NodeProperty::builder()
          .display_name("表单描述")
          .name("form_description")
          .kind(NodePropertyKind::String)
          .required(false)
          .description("表单的描述信息")
          .placeholder("请填写以下信息")
          .build(),
        NodeProperty::builder()
          .display_name("重定向 URL")
          .name("form_redirect_url")
          .kind(NodePropertyKind::String)
          .required(false)
          .description("表单提交后重定向的 URL")
          .placeholder("https://example.com/success")
          .build(),
        // 时间限制配置
        NodeProperty::builder()
          .display_name("启用时间限制")
          .name("webhook_limit_wait_time")
          .kind(NodePropertyKind::Boolean)
          .required(false)
          .description("是否限制 Webhook 等待的时间")
          .value(json!(false))
          .build(),
        NodeProperty::builder()
          .display_name("启用时间限制")
          .name("form_limit_wait_time")
          .kind(NodePropertyKind::Boolean)
          .required(false)
          .description("是否限制表单等待的时间")
          .value(json!(false))
          .build(),
      ]);

    let definition = base.build()?;

    Ok(Self { definition: Arc::new(definition) })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::core::WaitNode;
  use crate::core::wait_node::FormFieldType;
  use hetumind_core::workflow::Node;

  #[test]
  fn test_node_definition_properties() {
    let node = WaitNode::new().unwrap();
    let definition = node.default_node_executor().unwrap().definition();

    // 验证基本属性
    assert_eq!(definition.version, Version::new(1, 0, 0));
    assert_eq!(definition.inputs.len(), 1);
    assert_eq!(definition.outputs.len(), 1);

    // 验证属性配置
    let wait_mode_prop = definition.properties.iter().find(|p| p.name == "wait_mode");
    assert!(wait_mode_prop.is_some());
    assert!(wait_mode_prop.unwrap().required);

    let time_amount_prop = definition.properties.iter().find(|p| p.name == "time_amount");
    assert!(time_amount_prop.is_some());
    assert!(!time_amount_prop.unwrap().required);

    let form_title_prop = definition.properties.iter().find(|p| p.name == "form_title");
    assert!(form_title_prop.is_some());
    assert!(!form_title_prop.unwrap().required);
  }

  #[test]
  fn test_http_method_string_conversion() {
    // 测试 HTTP 方法的字符串表示
    let get_json = serde_json::to_string(&HttpMethod::Get).unwrap();
    assert_eq!(get_json, "\"GET\"");

    let post_json = serde_json::to_string(&HttpMethod::Post).unwrap();
    assert_eq!(post_json, "\"POST\"");
  }

  #[test]
  fn test_response_mode_string_conversion() {
    // 测试响应模式的字符串表示
    let on_received_json = serde_json::to_string(&ResponseMode::OnReceived).unwrap();
    assert_eq!(on_received_json, "\"on_received\"");

    let on_finished_json = serde_json::to_string(&ResponseMode::OnExecutionFinished).unwrap();
    assert_eq!(on_finished_json, "\"on_execution_finished\"");

    let never_json = serde_json::to_string(&ResponseMode::Never).unwrap();
    assert_eq!(never_json, "\"never\"");
  }

  #[test]
  fn test_wait_mode_string_conversion() {
    // 测试等待模式的字符串表示
    let interval_json = serde_json::to_string(&WaitMode::TimeInterval).unwrap();
    assert_eq!(interval_json, "\"time_interval\"");

    let specific_json = serde_json::to_string(&WaitMode::SpecificTime).unwrap();
    assert_eq!(specific_json, "\"specific_time\"");

    let webhook_json = serde_json::to_string(&WaitMode::Webhook).unwrap();
    assert_eq!(webhook_json, "\"webhook\"");

    let form_json = serde_json::to_string(&WaitMode::Form).unwrap();
    assert_eq!(form_json, "\"form\"");
  }

  #[test]
  fn test_time_unit_string_conversion() {
    // 测试时间单位的字符串表示
    let seconds_json = serde_json::to_string(&TimeUnit::Seconds).unwrap();
    assert_eq!(seconds_json, "\"seconds\"");

    let minutes_json = serde_json::to_string(&TimeUnit::Minutes).unwrap();
    assert_eq!(minutes_json, "\"minutes\"");

    let hours_json = serde_json::to_string(&TimeUnit::Hours).unwrap();
    assert_eq!(hours_json, "\"hours\"");

    let days_json = serde_json::to_string(&TimeUnit::Days).unwrap();
    assert_eq!(days_json, "\"days\"");
  }

  #[test]
  fn test_authentication_type_string_conversion() {
    // 测试认证类型的字符串表示
    let none_json = serde_json::to_string(&AuthenticationType::None).unwrap();
    assert_eq!(none_json, "\"none\"");

    let basic_json = serde_json::to_string(&AuthenticationType::BasicAuth).unwrap();
    assert_eq!(basic_json, "\"basic_auth\"");

    let header_json = serde_json::to_string(&AuthenticationType::HeaderAuth).unwrap();
    assert_eq!(header_json, "\"header_auth\"");
  }

  #[test]
  fn test_limit_type_string_conversion() {
    // 测试限制类型的字符串表示
    let interval_json = serde_json::to_string(&LimitType::AfterTimeInterval).unwrap();
    assert_eq!(interval_json, "\"after_time_interval\"");

    let specific_json = serde_json::to_string(&LimitType::AtSpecifiedTime).unwrap();
    assert_eq!(specific_json, "\"at_specified_time\"");
  }

  #[test]
  fn test_form_field_type_string_conversion() {
    // 测试表单字段类型的字符串表示
    let text_json = serde_json::to_string(&FormFieldType::Text).unwrap();
    assert_eq!(text_json, "\"text\"");

    let textarea_json = serde_json::to_string(&FormFieldType::Textarea).unwrap();
    assert_eq!(textarea_json, "\"textarea\"");

    let select_json = serde_json::to_string(&FormFieldType::Select).unwrap();
    assert_eq!(select_json, "\"select\"");
  }
}
