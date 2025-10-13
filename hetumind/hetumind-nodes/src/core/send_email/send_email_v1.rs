use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::{
  version::Version,
  workflow::{
    ConnectionKind, ExecutionDataItems, ExecutionDataMap, InputPortConfig, NodeDefinition, NodeDefinitionBuilder,
    NodeExecutable, NodeExecutionContext, NodeExecutionError, NodeProperty, NodePropertyKind, OutputPortConfig,
    RegistrationError, make_execution_data_map,
  },
};
use serde_json::json;

use super::{
  EmailFormat, EmailPriority, SendEmailConfig, SmtpConfig, SmtpSecurity,
  utils::{SendEmailResult, send_email_async, send_email_sync},
};

#[derive(Debug)]
pub struct SendEmailV1 {
  pub definition: Arc<NodeDefinition>,
}

impl SendEmailV1 {
  /// 处理邮件发送结果
  fn handle_send_result(
    &self,
    result: SendEmailResult,
    continue_on_fail: bool,
  ) -> Result<serde_json::Value, NodeExecutionError> {
    if result.success {
      Ok(json!({
        "success": true,
        "message_id": result.message_id,
        "sent_at": result.sent_at.to_rfc3339(),
        "status": "sent"
      }))
    } else if continue_on_fail {
      log::warn!("邮件发送失败，但继续执行: {:?}", result.error);
      Ok(json!({
        "success": false,
        "error": result.error,
        "sent_at": result.sent_at.to_rfc3339(),
        "status": "failed"
      }))
    } else {
      Err(NodeExecutionError::ExecutionFailed { node_name: "SendEmailNode".to_string().into(), message: result.error })
    }
  }

  /// 构建邮件配置
  fn build_email_config(
    &self,
    context: &NodeExecutionContext,
    item_index: usize,
  ) -> Result<SendEmailConfig, NodeExecutionError> {
    let node = context.current_node()?;

    // SMTP 配置
    let smtp_config = SmtpConfig {
      host: node.get_parameter("smtp_host")?,
      port: node.get_parameter("smtp_port")?,
      security: node.get_parameter("smtp_security")?,
      username: node.get_parameter("smtp_username")?,
      password: node.get_parameter("smtp_password")?,
      from_email: node.get_parameter("from_email")?,
      from_name: node
        .get_optional_parameter("from_name")
        .ok_or_else(|| NodeExecutionError::InvalidInput("from_name parameter is invalid".to_string()))?,
      connection_timeout: node
        .get_optional_parameter("connection_timeout")
        .ok_or_else(|| NodeExecutionError::InvalidInput("connection_timeout parameter is invalid".to_string()))?,
      allow_unauthorized_certs: node
        .get_optional_parameter("allow_unauthorized_certs")
        .ok_or_else(|| NodeExecutionError::InvalidInput("allow_unauthorized_certs parameter is invalid".to_string()))?,
    };

    // 验证 SMTP 配置
    smtp_config.validate().map_err(|e| NodeExecutionError::ExecutionFailed {
      node_name: "SendEmailNode".to_string().into(),
      message: Some(format!("Invalid SMTP configuration: {}", e)),
    })?;

    // 邮件配置
    let email_format = node.get_parameter("email_format")?;
    let subject = self.process_template_field(&node.get_parameter::<String>("subject")?, context, item_index)?;
    let to_emails = self.process_template_field(&node.get_parameter::<String>("to_emails")?, context, item_index)?;
    let cc_emails = node
      .get_optional_parameter::<String>("cc_emails")
      .map(|emails| self.process_template_field(&emails, context, item_index))
      .transpose()?;
    let bcc_emails = node
      .get_optional_parameter::<String>("bcc_emails")
      .map(|emails| self.process_template_field(&emails, context, item_index))
      .transpose()?;
    let reply_to = node
      .get_optional_parameter::<String>("reply_to")
      .map(|email| self.process_template_field(&email, context, item_index))
      .transpose()?;
    let priority = node.get_optional_parameter::<EmailPriority>("priority");
    let continue_on_fail = node.get_optional_parameter("continue_on_fail").unwrap_or(true);
    let append_attribution = node.get_optional_parameter("append_attribution").unwrap_or(false);

    // 处理邮件内容
    let (text_content, html_content) = match email_format {
      EmailFormat::Text => {
        let content = node.get_parameter::<String>("text_content")?;
        let processed = self.process_template_field(&content, context, item_index)?;
        (Some(processed), None)
      }
      EmailFormat::Html => {
        let content = node.get_parameter::<String>("html_content")?;
        let processed = self.process_template_field(&content, context, item_index)?;
        (None, Some(processed))
      }
      EmailFormat::Both => {
        let text = node.get_parameter::<String>("text_content")?;
        let html = node.get_parameter::<String>("html_content")?;
        let processed_text = self.process_template_field(&text, context, item_index)?;
        let processed_html = self.process_template_field(&html, context, item_index)?;
        (Some(processed_text), Some(processed_html))
      }
    };

    // 处理附件配置
    let attachments = if let Some(attachment_fields) = node.get_optional_parameter::<Vec<String>>("attachment_fields") {
      let mut attachment_configs = Vec::new();
      for field_name in attachment_fields {
        attachment_configs.push(super::AttachmentConfig { field_name, custom_filename: None, content_type: None });
      }
      Some(attachment_configs)
    } else {
      None
    };

    // 处理自定义邮件头
    let custom_headers = node
      .get_optional_parameter("custom_headers")
      .ok_or_else(|| NodeExecutionError::InvalidInput("custom_headers parameter is invalid".to_string()))?;

    let config = SendEmailConfig {
      smtp_config,
      email_format,
      subject,
      to_emails,
      cc_emails,
      bcc_emails,
      reply_to,
      priority,
      attachments,
      text_content,
      html_content,
      continue_on_fail: Some(continue_on_fail),
      append_attribution: Some(append_attribution),
      custom_headers,
    };

    // 验证邮件配置
    config.validate().map_err(|e| NodeExecutionError::ExecutionFailed {
      node_name: "SendEmailNode".to_string().into(),
      message: Some(format!("Invalid email configuration: {}", e)),
    })?;

    Ok(config)
  }

  /// 处理模板字段，支持变量替换
  fn process_template_field(
    &self,
    field: &str,
    context: &NodeExecutionContext,
    item_index: usize,
  ) -> Result<String, NodeExecutionError> {
    // 获取输入数据
    let input_data = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      input_data[item_index].clone()
    } else {
      return Err(NodeExecutionError::ExecutionFailed {
        node_name: "SendEmailNode".to_string().into(),
        message: Some("No input data available for template processing".to_string()),
      });
    };

    // 如果字段包含模板变量，进行替换
    if field.contains("{{") && field.contains("}}") {
      super::utils::process_template_variables(field, &input_data)
    } else {
      Ok(field.to_string())
    }
  }

  /// 执行邮件发送
  async fn execute_email_send(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    log::info!(
      "[DEBUG] 开始执行 Send Email 节点 workflow_id:{}, node_name:{}, node_kind:{}",
      context.workflow.id,
      node.name,
      node.kind
    );

    // 获取输入数据
    let input_items = if let Some(input_collection) = context.get_input_items(ConnectionKind::Main, 0)
      && let ExecutionDataItems::Items(input_data) = input_collection
    {
      log::info!("Send Email 节点接收到 {} 个输入项", input_data.len());
      input_data
    } else {
      log::error!("Send Email 节点没有接收到输入数据");
      return Ok(make_execution_data_map(vec![(
        ConnectionKind::Main,
        vec![ExecutionDataItems::new_items(Default::default())],
      )]));
    };

    // 获取发送模式配置
    let use_async = node.get_optional_parameter("use_async").unwrap_or(false);
    let max_concurrent = node.get_optional_parameter("max_concurrent").unwrap_or(5);
    let test_connection_only = node.get_optional_parameter("test_connection_only").unwrap_or(false);

    // 如果只是测试连接，执行测试并返回
    if test_connection_only {
      log::info!("执行 SMTP 连接测试");
      return self.test_smtp_connection(context).await;
    }

    let mut results = Vec::new();

    if use_async && input_items.len() > 1 {
      // 异步批量发送
      log::info!("使用异步模式发送 {} 封邮件，最大并发数: {}", input_items.len(), max_concurrent);

      // 使用信号量控制并发数
      let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
      let mut tasks = Vec::new();

      for (index, input_item) in input_items.iter().enumerate() {
        let semaphore = semaphore.clone();

        let config = self.build_email_config(context, index)?;
        let input_item = input_item.clone();

        let task = async move {
          let _permit = semaphore.acquire().await.unwrap();
          log::debug!("开始异步发送邮件项: {}", index);

          match send_email_async(&config, &input_item, context.binary_data_manager.clone()).await {
            Ok(result) => {
              log::debug!("异步发送邮件项 {} 完成: {}", index, result.success);
              (index, result)
            }
            Err(e) => {
              log::error!("异步发送邮件项 {} 失败: {}", index, e);
              (
                index,
                SendEmailResult {
                  success: false,
                  message_id: None,
                  error: Some(format!("Async send failed: {}", e)),
                  sent_at: chrono::Utc::now(),
                },
              )
            }
          }
        };

        tasks.push(task);
      }

      // 等待所有任务完成
      let task_results = futures::future::join_all(tasks).await;

      // 按原始顺序重新排列结果
      let mut sorted_results = vec![
        SendEmailResult {
          success: false,
          message_id: None,
          error: Some("No result".to_string()),
          sent_at: chrono::Utc::now(),
        };
        task_results.len()
      ];

      for (index, result) in task_results {
        if index < sorted_results.len() {
          sorted_results[index] = result;
        }
      }

      // 处理结果
      for (index, result) in sorted_results.into_iter().enumerate() {
        let continue_on_fail = self.build_email_config(context, index)?.continue_on_fail.unwrap_or(true);
        let result_json = self.handle_send_result(result, continue_on_fail)?;

        let mut output_json = input_items[index].json().clone();
        if let serde_json::Value::Object(ref mut map) = output_json {
          map.insert("email_result".to_string(), result_json);
        }

        // Convert JSON to ExecutionData
        let output_data = hetumind_core::workflow::ExecutionData::new_json(output_json, None);
        results.push(output_data);
      }
    } else {
      // 同步逐个发送
      log::info!("使用同步模式发送 {} 封邮件", input_items.len());

      for (index, input_item) in input_items.iter().enumerate() {
        log::debug!("开始同步发送邮件项: {}", index);

        match self.build_email_config(context, index) {
          Ok(config) => {
            let continue_on_fail = config.continue_on_fail.unwrap_or(true);

            match send_email_sync(&config, input_item, &context.binary_data_manager).await {
              Ok(result) => {
                log::debug!("同步发送邮件项 {} 完成: {}", index, result.success);

                match self.handle_send_result(result, continue_on_fail) {
                  Ok(result_json) => {
                    let mut output_json = input_item.json().clone();
                    if let serde_json::Value::Object(ref mut map) = output_json {
                      map.insert("email_result".to_string(), result_json);
                    }
                    let output_data = hetumind_core::workflow::ExecutionData::new_json(output_json, None);
                    results.push(output_data);
                  }
                  Err(e) => {
                    if continue_on_fail {
                      log::warn!("处理邮件项 {} 结果失败，但继续执行: {}", index, e);
                      let mut output_json = input_item.json().clone();
                      if let serde_json::Value::Object(ref mut map) = output_json {
                        map.insert(
                          "email_result".to_string(),
                          json!({
                            "success": false,
                            "error": format!("Result processing failed: {}", e),
                            "status": "failed"
                          }),
                        );
                      }
                      let output_data = hetumind_core::workflow::ExecutionData::new_json(output_json, None);
                      results.push(output_data);
                    } else {
                      return Err(e);
                    }
                  }
                }
              }
              Err(e) => {
                log::error!("同步发送邮件项 {} 失败: {}", index, e);

                let continue_on_fail = config.continue_on_fail.unwrap_or(true);
                if continue_on_fail {
                  let mut output_json = input_item.json().clone();
                  if let serde_json::Value::Object(ref mut map) = output_json {
                    map.insert(
                      "email_result".to_string(),
                      json!({
                        "success": false,
                        "error": format!("Send failed: {}", e),
                        "status": "failed"
                      }),
                    );
                  }
                  let output_data = hetumind_core::workflow::ExecutionData::new_json(output_json, None);
                  results.push(output_data);
                } else {
                  return Err(e);
                }
              }
            }
          }
          Err(e) => {
            log::error!("构建邮件项 {} 配置失败: {}", index, e);
            return Err(e);
          }
        }
      }
    }

    log::info!("Send Email 节点执行完成，处理了 {} 个项目", results.len());

    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(results)])]))
  }

  /// 测试 SMTP 连接
  async fn test_smtp_connection(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;

    // 构建 SMTP 配置 (只需要基本的连接信息)
    let smtp_config = SmtpConfig {
      host: node.get_parameter("smtp_host")?,
      port: node.get_parameter("smtp_port")?,
      security: node.get_parameter("smtp_security")?,
      username: node.get_parameter("smtp_username")?,
      password: node.get_parameter("smtp_password")?,
      from_email: node.get_parameter("from_email")?,
      from_name: node
        .get_optional_parameter("from_name")
        .ok_or_else(|| NodeExecutionError::InvalidInput("from_name parameter is invalid".to_string()))?,
      connection_timeout: node
        .get_optional_parameter("connection_timeout")
        .ok_or_else(|| NodeExecutionError::InvalidInput("connection_timeout parameter is invalid".to_string()))?,
      allow_unauthorized_certs: node
        .get_optional_parameter("allow_unauthorized_certs")
        .ok_or_else(|| NodeExecutionError::InvalidInput("allow_unauthorized_certs parameter is invalid".to_string()))?,
    };

    // 验证配置
    smtp_config.validate().map_err(|e| NodeExecutionError::ExecutionFailed {
      node_name: "SendEmailNode".to_string().into(),
      message: Some(format!("Invalid SMTP configuration: {}", e)),
    })?;

    // 执行连接测试
    match super::utils::test_smtp_connection(&smtp_config).await {
      Ok(success) => {
        let result_json = json!({
          "connection_test": success,
          "smtp_host": smtp_config.host,
          "smtp_port": smtp_config.port,
          "smtp_security": format!("{:?}", smtp_config.security),
          "username": smtp_config.username,
          "from_email": smtp_config.from_email,
          "test_time": chrono::Utc::now().to_rfc3339(),
          "message": if success {
            "SMTP connection test successful"
          } else {
            "SMTP connection test failed"
          }
        });

        let result_data = hetumind_core::workflow::ExecutionData::new_json(result_json, None);

        Ok(make_execution_data_map(vec![(
          ConnectionKind::Main,
          vec![ExecutionDataItems::new_items(vec![result_data])],
        )]))
      }
      Err(e) => {
        let error_json = json!({
          "connection_test": false,
          "error": format!("Connection test failed: {}", e),
          "smtp_host": smtp_config.host,
          "smtp_port": smtp_config.port,
          "test_time": chrono::Utc::now().to_rfc3339()
        });

        let error_data = hetumind_core::workflow::ExecutionData::new_json(error_json, None);

        Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![error_data])])]))
      }
    }
  }
}

/// Send Email 邮件发送节点 V1
///
/// 用于通过 SMTP 协议发送电子邮件。支持多种邮件格式、附件处理、多收件人管理等功能。
///
/// # 支持的功能
/// - **多种邮件格式**: 纯文本、HTML 或混合格式
/// - **多收件人**: 支持主收件人、抄送（CC）、密送（BCC）
/// - **附件支持**: 从输入数据的二进制字段添加附件
/// - **SMTP 配置**: 完整的 SMTP 服务器配置和认证
/// - **模板变量**: 支持在邮件内容中使用变量替换
/// - **异步发送**: 支持批量异步发送提高性能
/// - **错误处理**: 可配置失败时是否继续执行
///
/// # 输入数据
/// - 邮件内容可以使用模板变量，格式: `{{field_name}}`
/// - 附件数据需要存储在输入数据的二进制字段中
///
/// # 输出数据
/// - 原始输入数据
/// - `email_result`: 包含发送状态、消息ID、错误信息等
#[async_trait]
impl NodeExecutable for SendEmailV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    self.execute_email_send(context).await
  }
}

impl TryFrom<NodeDefinitionBuilder> for SendEmailV1 {
  type Error = RegistrationError;

  fn try_from(mut base: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    base
      .version(Version::new(1, 0, 0))
      .inputs([InputPortConfig::builder().kind(ConnectionKind::Main).display_name("Input").build()])
      .outputs([OutputPortConfig::builder().kind(ConnectionKind::Main).display_name("Output").build()])
      .properties([
        // SMTP 配置
        NodeProperty::builder()
          .display_name("SMTP Host".to_string())
          .name("smtp_host")
          .required(true)
          .description("SMTP server hostname (e.g., smtp.gmail.com)".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("smtp.gmail.com".to_string())
          .build(),
        NodeProperty::builder()
          .display_name("SMTP Port".to_string())
          .name("smtp_port")
          .required(true)
          .description("SMTP server port (e.g., 587 for STARTTLS, 465 for SSL)".to_string())
          .kind(NodePropertyKind::Number)
          .value(json!(587))
          .build(),
        NodeProperty::builder()
          .display_name("SMTP Security".to_string())
          .name("smtp_security")
          .required(true)
          .description("Connection security type".to_string())
          .kind(NodePropertyKind::Options)
          .value(json!(SmtpSecurity::Starttls))
          .options(vec![
            Box::new(NodeProperty::new_option(
              "STARTTLS",
              "starttls",
              json!(SmtpSecurity::Starttls),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option("SSL/TLS", "ssl", json!(SmtpSecurity::Ssl), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("None", "none", json!(SmtpSecurity::None), NodePropertyKind::String)),
          ])
          .build(),
        NodeProperty::builder()
          .display_name("SMTP Username".to_string())
          .name("smtp_username")
          .required(true)
          .description("SMTP authentication username".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("your-email@gmail.com".to_string())
          .build(),
        NodeProperty::builder()
          .display_name("SMTP Password".to_string())
          .name("smtp_password")
          .required(true)
          .description("SMTP authentication password or app password".to_string())
          .kind(NodePropertyKind::String)
          .password(true)
          .build(),
        // 发件人配置
        NodeProperty::builder()
          .display_name("From Email".to_string())
          .name("from_email")
          .required(true)
          .description("Sender email address".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("sender@example.com".to_string())
          .build(),
        NodeProperty::builder()
          .display_name("From Name".to_string())
          .name("from_name")
          .required(false)
          .description("Sender display name (optional)".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("Your Name".to_string())
          .build(),
        // 收件人配置
        NodeProperty::builder()
          .display_name("To Emails".to_string())
          .name("to_emails")
          .required(true)
          .description("Recipient email addresses (comma-separated)".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("recipient1@example.com, recipient2@example.com".to_string())
          .build(),
        NodeProperty::builder()
          .display_name("CC Emails".to_string())
          .name("cc_emails")
          .required(false)
          .description("CC recipient email addresses (comma-separated, optional)".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("cc1@example.com, cc2@example.com".to_string())
          .build(),
        NodeProperty::builder()
          .display_name("BCC Emails".to_string())
          .name("bcc_emails")
          .required(false)
          .description("BCC recipient email addresses (comma-separated, optional)".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("bcc1@example.com, bcc2@example.com".to_string())
          .build(),
        NodeProperty::builder()
          .display_name("Reply To".to_string())
          .name("reply_to")
          .required(false)
          .description("Reply-to email address (optional)".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("reply@example.com".to_string())
          .build(),
        // 邮件内容配置
        NodeProperty::builder()
          .display_name("Subject".to_string())
          .name("subject")
          .required(true)
          .description("Email subject line".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("Your Subject Here".to_string())
          .build(),
        NodeProperty::builder()
          .display_name("Email Format".to_string())
          .name("email_format")
          .required(true)
          .description("Email content format".to_string())
          .kind(NodePropertyKind::Options)
          .value(json!(EmailFormat::Both))
          .options(vec![
            Box::new(NodeProperty::new_option("Text Only", "text", json!(EmailFormat::Text), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("HTML Only", "html", json!(EmailFormat::Html), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option(
              "Text and HTML",
              "both",
              json!(EmailFormat::Both),
              NodePropertyKind::String,
            )),
          ])
          .build(),
        NodeProperty::builder()
          .display_name("Text Content".to_string())
          .name("text_content")
          .required(false)
          .description("Plain text email content (required for Text or Both format)".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("Hello, this is the plain text version of the email.".to_string())
          .build(),
        NodeProperty::builder()
          .display_name("HTML Content".to_string())
          .name("html_content")
          .required(false)
          .description("HTML email content (required for HTML or Both format)".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("<h1>Hello</h1><p>This is the <strong>HTML</strong> version of the email.</p>".to_string())
          .build(),
        // 附件配置
        NodeProperty::builder()
          .display_name("Attachment Fields".to_string())
          .name("attachment_fields")
          .required(false)
          .description("Binary field names from input data to attach as files".to_string())
          .kind(NodePropertyKind::String)
          .placeholder("attachment1, attachment2".to_string())
          .build(),
        // 选项配置
        NodeProperty::builder()
          .display_name("Priority".to_string())
          .name("priority")
          .required(false)
          .description("Email priority level".to_string())
          .kind(NodePropertyKind::Options)
          .value(json!(EmailPriority::Normal))
          .options(vec![
            Box::new(NodeProperty::new_option("Low", "low", json!(EmailPriority::Low), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option(
              "Normal",
              "normal",
              json!(EmailPriority::Normal),
              NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option("High", "high", json!(EmailPriority::High), NodePropertyKind::String)),
          ])
          .build(),
        NodeProperty::builder()
          .display_name("Continue on Fail".to_string())
          .name("continue_on_fail")
          .required(false)
          .description("Continue workflow execution even if email sending fails".to_string())
          .kind(NodePropertyKind::Boolean)
          .value(json!(true))
          .build(),
        NodeProperty::builder()
          .display_name("Append Attribution".to_string())
          .name("append_attribution")
          .required(false)
          .description("Add attribution header to identify the email was sent by HetuMind".to_string())
          .kind(NodePropertyKind::Boolean)
          .value(json!(false))
          .build(),
        // 高级配置
        NodeProperty::builder()
          .display_name("Connection Timeout".to_string())
          .name("connection_timeout")
          .required(false)
          .description("SMTP connection timeout in seconds".to_string())
          .kind(NodePropertyKind::Number)
          .value(json!(30))
          .build(),
        NodeProperty::builder()
          .display_name("Allow Unauthorized Certs".to_string())
          .name("allow_unauthorized_certs")
          .required(false)
          .description("Allow unauthorized SSL certificates (for testing only)".to_string())
          .kind(NodePropertyKind::Boolean)
          .value(json!(false))
          .build(),
        NodeProperty::builder()
          .display_name("Use Async Sending".to_string())
          .name("use_async")
          .required(false)
          .description("Use async sending for multiple emails (improves performance)".to_string())
          .kind(NodePropertyKind::Boolean)
          .value(json!(false))
          .build(),
        NodeProperty::builder()
          .display_name("Max Concurrent".to_string())
          .name("max_concurrent")
          .required(false)
          .description("Maximum concurrent connections when using async sending".to_string())
          .kind(NodePropertyKind::Number)
          .value(json!(5))
          .build(),
        NodeProperty::builder()
          .display_name("Custom Headers".to_string())
          .name("custom_headers")
          .required(false)
          .description("Custom email headers as JSON object".to_string())
          .kind(NodePropertyKind::String)
          .placeholder(r#"{"X-Custom-Header": "value"}"#.to_string())
          .build(),
        NodeProperty::builder()
          .display_name("Test Connection Only".to_string())
          .name("test_connection_only")
          .required(false)
          .description("Only test SMTP connection without sending emails".to_string())
          .kind(NodePropertyKind::Boolean)
          .value(json!(false))
          .build(),
      ]);

    let definition = base.build()?;

    Ok(Self { definition: Arc::new(definition) })
  }
}
