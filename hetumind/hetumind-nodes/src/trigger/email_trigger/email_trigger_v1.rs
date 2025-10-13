use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::types::JsonValue;
use hetumind_core::version::Version;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, NodeDefinition, NodeDefinitionBuilder, NodeExecutable,
  NodeExecutionContext, NodeExecutionError, NodeGroupKind, NodeProperty, RegistrationError, make_execution_data_map,
};

use crate::constants::EMAIL_TRIGGER_NODE_KIND;

use super::{EmailProcessingConfig, EmailReadFormat, EmailTriggerConfig, ImapAuthentication, ImapSecurity};

pub struct EmailTriggerV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinitionBuilder> for EmailTriggerV1 {
  type Error = RegistrationError;

  fn try_from(builder: NodeDefinitionBuilder) -> Result<Self, Self::Error> {
    let definition = builder.build()?;
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl NodeExecutable for EmailTriggerV1 {
  fn definition(&self) -> Arc<NodeDefinition> {
    self.definition.clone()
  }

  async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
    let node = context.current_node()?;
    let config = self.parse_config(&serde_json::Value::Object(node.parameters.clone().into_inner()))?;

    if !config.enabled {
      return Err(NodeExecutionError::ParameterValidation(
        hetumind_core::workflow::ValidationError::NodePropertyValidation("Email trigger is disabled".to_string()),
      ));
    }

    // Email 触发器作为入口点，返回空数据
    // 实际的邮件监控和处理在触发器框架层面完成
    Ok(make_execution_data_map(vec![(ConnectionKind::Main, vec![ExecutionDataItems::new_items(vec![])])]))
  }
}

impl EmailTriggerV1 {
  /// 解析节点参数为配置结构
  fn parse_config(&self, parameters: &JsonValue) -> Result<EmailTriggerConfig, NodeExecutionError> {
    let imap_connection = self.parse_imap_connection(parameters)?;
    let processing = self.parse_email_processing(parameters)?;
    let poll_interval = self.parse_poll_interval(parameters)?;
    let enabled = self.parse_enabled(parameters)?;

    let config = EmailTriggerConfig { imap_connection, processing, poll_interval, enabled };

    config.validate().map_err(|e| {
      NodeExecutionError::ParameterValidation(hetumind_core::workflow::ValidationError::NodePropertyValidation(
        format!("Invalid email trigger configuration: {}", e),
      ))
    })?;

    Ok(config)
  }

  /// 解析 IMAP 连接配置
  fn parse_imap_connection(&self, parameters: &JsonValue) -> Result<super::ImapConnectionConfig, NodeExecutionError> {
    let host = parameters
      .get("imap_host")
      .and_then(|v| v.as_str())
      .ok_or_else(|| {
        NodeExecutionError::ParameterValidation(hetumind_core::workflow::ValidationError::RequiredFieldMissing {
          field: "imap_host".to_string(),
        })
      })?
      .to_string();

    let port = parameters.get("imap_port").and_then(|v| v.as_u64()).ok_or_else(|| {
      NodeExecutionError::ParameterValidation(hetumind_core::workflow::ValidationError::RequiredFieldMissing {
        field: "imap_port".to_string(),
      })
    })? as u16;

    let security_str = parameters.get("imap_security").and_then(|v| v.as_str()).unwrap_or("ssl");
    let security = match security_str {
      "ssl" => ImapSecurity::Ssl,
      "starttls" => ImapSecurity::Starttls,
      "none" => ImapSecurity::None,
      _ => {
        return Err(NodeExecutionError::ParameterValidation(
          hetumind_core::workflow::ValidationError::NodePropertyValidation(format!(
            "Invalid security mode: {}",
            security_str
          )),
        ));
      }
    };

    let auth_str = parameters.get("imap_authentication").and_then(|v| v.as_str()).unwrap_or("normal");
    let authentication = match auth_str {
      "normal" => ImapAuthentication::Normal,
      "oauth" => ImapAuthentication::OAuth,
      "cram_md5" => ImapAuthentication::CramMd5,
      "digest_md5" => ImapAuthentication::DigestMd5,
      _ => {
        return Err(NodeExecutionError::ParameterValidation(
          hetumind_core::workflow::ValidationError::NodePropertyValidation(format!(
            "Invalid authentication method: {}",
            auth_str
          )),
        ));
      }
    };

    let username = parameters
      .get("imap_username")
      .and_then(|v| v.as_str())
      .ok_or_else(|| {
        NodeExecutionError::ParameterValidation(hetumind_core::workflow::ValidationError::RequiredFieldMissing {
          field: "imap_username".to_string(),
        })
      })?
      .to_string();

    let password = parameters
      .get("imap_password")
      .and_then(|v| v.as_str())
      .ok_or_else(|| {
        NodeExecutionError::ParameterValidation(hetumind_core::workflow::ValidationError::RequiredFieldMissing {
          field: "imap_password".to_string(),
        })
      })?
      .to_string();

    let mailbox = parameters.get("imap_mailbox").and_then(|v| v.as_str()).map(|s| s.to_string());
    let connection_timeout = parameters.get("connection_timeout").and_then(|v| v.as_u64()).map(|v| v as u64);
    let read_timeout = parameters.get("read_timeout").and_then(|v| v.as_u64()).map(|v| v as u64);

    Ok(super::ImapConnectionConfig {
      host,
      port,
      security,
      authentication,
      username,
      password,
      mailbox,
      connection_timeout,
      read_timeout,
    })
  }

  /// 解析邮件处理配置
  fn parse_email_processing(&self, parameters: &JsonValue) -> Result<EmailProcessingConfig, NodeExecutionError> {
    let format_str = parameters.get("email_read_format").and_then(|v| v.as_str()).unwrap_or("resolved");
    let read_format = match format_str {
      "raw" => EmailReadFormat::Raw,
      "resolved" => EmailReadFormat::Resolved,
      "simple" => EmailReadFormat::Simple,
      _ => {
        return Err(NodeExecutionError::ParameterValidation(
          hetumind_core::workflow::ValidationError::NodePropertyValidation(format!(
            "Invalid email read format: {}",
            format_str
          )),
        ));
      }
    };

    let filter = self.parse_email_filter(parameters)?;
    let attachment_options = self.parse_attachment_options(parameters)?;
    let max_emails = parameters.get("max_emails").and_then(|v| v.as_u64()).map(|v| v as u32);
    let mark_as_read = parameters.get("mark_as_read").and_then(|v| v.as_bool());
    let delete_after_read = parameters.get("delete_after_read").and_then(|v| v.as_bool());

    Ok(EmailProcessingConfig { read_format, filter, attachment_options, max_emails, mark_as_read, delete_after_read })
  }

  /// 解析邮件过滤条件
  fn parse_email_filter(&self, parameters: &JsonValue) -> Result<Option<super::EmailFilter>, NodeExecutionError> {
    let filter_enabled = parameters.get("enable_filter").and_then(|v| v.as_bool()).unwrap_or(false);

    if !filter_enabled {
      return Ok(None);
    }

    let from_filter = parameters.get("from_filter").and_then(|v| v.as_str()).map(|s| s.to_string());
    let to_filter = parameters.get("to_filter").and_then(|v| v.as_str()).map(|s| s.to_string());
    let subject_filter = parameters.get("subject_filter").and_then(|v| v.as_str()).map(|s| s.to_string());
    let content_filter = parameters.get("content_filter").and_then(|v| v.as_str()).map(|s| s.to_string());
    let read_only_unread = parameters.get("read_only_unread").and_then(|v| v.as_bool());
    let has_attachments = parameters.get("has_attachments_filter").and_then(|v| v.as_bool());

    Ok(Some(super::EmailFilter {
      from_filter,
      to_filter,
      subject_filter,
      content_filter,
      read_only_unread,
      has_attachments,
    }))
  }

  /// 解析附件处理选项
  fn parse_attachment_options(
    &self,
    parameters: &JsonValue,
  ) -> Result<Option<super::AttachmentOptions>, NodeExecutionError> {
    let download_attachments = parameters.get("download_attachments").and_then(|v| v.as_bool()).unwrap_or(false);

    if !download_attachments {
      return Ok(None);
    }

    let max_attachment_size = parameters.get("max_attachment_size").and_then(|v| v.as_u64()).map(|v| v as u64);

    let allowed_types = parameters
      .get("allowed_file_types")
      .and_then(|v| v.as_array())
      .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());

    let forbidden_types = parameters
      .get("forbidden_file_types")
      .and_then(|v| v.as_array())
      .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());

    Ok(Some(super::AttachmentOptions {
      download_attachments: Some(download_attachments),
      max_attachment_size,
      allowed_file_types: allowed_types,
      forbidden_file_types: forbidden_types,
    }))
  }

  /// 解析轮询间隔
  fn parse_poll_interval(&self, parameters: &JsonValue) -> Result<u64, NodeExecutionError> {
    parameters
      .get("poll_interval")
      .and_then(|v| v.as_u64())
      .ok_or_else(|| {
        NodeExecutionError::ParameterValidation(hetumind_core::workflow::ValidationError::RequiredFieldMissing {
          field: "poll_interval".to_string(),
        })
      })
      .map(|v| v as u64)
  }

  /// 解析启用状态
  fn parse_enabled(&self, parameters: &JsonValue) -> Result<bool, NodeExecutionError> {
    Ok(parameters.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true))
  }
}

impl EmailTriggerV1 {
  /// 创建基础的节点定义
  pub fn create_base() -> NodeDefinitionBuilder {
    let mut base = NodeDefinitionBuilder::default();
    base
      .kind(EMAIL_TRIGGER_NODE_KIND)
      .version(Version::new(1, 0, 0))
      .groups([NodeGroupKind::Trigger])
      .display_name("Email Trigger")
      .description("Triggers workflow when new emails are received via IMAP")
      .outputs(vec![])
      .properties(vec![
        // IMAP Connection Section
        NodeProperty::builder()
          .display_name("IMAP Host")
          .name("imap_host")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(true)
          .description("IMAP server hostname (e.g., imap.gmail.com)")
          .hint("e.g., imap.gmail.com")
          .build(),
        NodeProperty::builder()
          .display_name("IMAP Port")
          .name("imap_port")
          .kind(hetumind_core::workflow::NodePropertyKind::Number)
          .required(true)
          .description("IMAP server port number")
          .value(JsonValue::Number(serde_json::Number::from(993)))
          .build(),
        NodeProperty::builder()
          .display_name("Security")
          .name("imap_security")
          .kind(hetumind_core::workflow::NodePropertyKind::Options)
          .options(vec![
            Box::new(NodeProperty::new_option(
              "SSL/TLS",
              "ssl",
              JsonValue::String("ssl".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "STARTTLS",
              "starttls",
              JsonValue::String("starttls".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "None",
              "none",
              JsonValue::String("none".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
          ])
          .required(true)
          .description("Connection security mode")
          .value(JsonValue::String("ssl".to_string()))
          .build(),
        NodeProperty::builder()
          .display_name("Authentication")
          .name("imap_authentication")
          .kind(hetumind_core::workflow::NodePropertyKind::Options)
          .options(vec![
            Box::new(NodeProperty::new_option(
              "Normal",
              "normal",
              JsonValue::String("normal".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "OAuth 2.0",
              "oauth",
              JsonValue::String("oauth".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "CRAM-MD5",
              "cram_md5",
              JsonValue::String("cram_md5".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "Digest MD5",
              "digest_md5",
              JsonValue::String("digest_md5".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
          ])
          .required(true)
          .description("IMAP authentication method")
          .value(JsonValue::String("normal".to_string()))
          .build(),
        NodeProperty::builder()
          .display_name("Username")
          .name("imap_username")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(true)
          .description("IMAP username")
          .hint("e.g., your-email@gmail.com")
          .build(),
        NodeProperty::builder()
          .display_name("Password")
          .name("imap_password")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(true)
          .description("IMAP password or access token")
          .hint("Use app-specific password for Gmail")
          .build(),
        NodeProperty::builder()
          .display_name("Mailbox")
          .name("imap_mailbox")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(false)
          .description("Mailbox name (default: INBOX)")
          .value(JsonValue::String("INBOX".to_string()))
          .build(),
        NodeProperty::builder()
          .display_name("Connection Timeout (seconds)")
          .name("connection_timeout")
          .kind(hetumind_core::workflow::NodePropertyKind::Number)
          .required(false)
          .description("Connection timeout in seconds")
          .value(JsonValue::Number(serde_json::Number::from(30)))
          .build(),
        NodeProperty::builder()
          .display_name("Read Timeout (seconds)")
          .name("read_timeout")
          .kind(hetumind_core::workflow::NodePropertyKind::Number)
          .required(false)
          .description("Read timeout in seconds")
          .value(JsonValue::Number(serde_json::Number::from(60)))
          .build(),
        // Email Processing Section
        NodeProperty::builder()
          .display_name("Email Read Format")
          .name("email_read_format")
          .kind(hetumind_core::workflow::NodePropertyKind::Options)
          .options(vec![
            Box::new(NodeProperty::new_option(
              "Raw",
              "raw",
              JsonValue::String("raw".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "Resolved",
              "resolved",
              JsonValue::String("resolved".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
            Box::new(NodeProperty::new_option(
              "Simple",
              "simple",
              JsonValue::String("simple".to_string()),
              hetumind_core::workflow::NodePropertyKind::String,
            )),
          ])
          .required(true)
          .description("Email output format")
          .value(JsonValue::String("resolved".to_string()))
          .build(),
        NodeProperty::builder()
          .display_name("Max Emails per Poll")
          .name("max_emails")
          .kind(hetumind_core::workflow::NodePropertyKind::Number)
          .required(false)
          .description("Maximum number of emails to process per poll")
          .value(JsonValue::Number(serde_json::Number::from(50)))
          .build(),
        NodeProperty::builder()
          .display_name("Mark as Read")
          .name("mark_as_read")
          .kind(hetumind_core::workflow::NodePropertyKind::Boolean)
          .required(false)
          .description("Mark processed emails as read")
          .value(JsonValue::Bool(true))
          .build(),
        NodeProperty::builder()
          .display_name("Delete After Read")
          .name("delete_after_read")
          .kind(hetumind_core::workflow::NodePropertyKind::Boolean)
          .required(false)
          .description("Delete emails after processing")
          .value(JsonValue::Bool(false))
          .build(),
        // Filter Section
        NodeProperty::builder()
          .display_name("Enable Filter")
          .name("enable_filter")
          .kind(hetumind_core::workflow::NodePropertyKind::Boolean)
          .required(false)
          .description("Enable email filtering")
          .value(JsonValue::Bool(false))
          .build(),
        NodeProperty::builder()
          .display_name("From Filter")
          .name("from_filter")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(false)
          .description("Filter by sender (supports wildcards)")
          .hint("e.g., noreply@*.com, support@company.com")
          .build(),
        NodeProperty::builder()
          .display_name("To Filter")
          .name("to_filter")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(false)
          .description("Filter by recipient (supports wildcards)")
          .hint("e.g., support@*.com")
          .build(),
        NodeProperty::builder()
          .display_name("Subject Filter")
          .name("subject_filter")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(false)
          .description("Filter by subject (supports wildcards)")
          .hint("e.g., *Invoice*, *Alert*")
          .build(),
        NodeProperty::builder()
          .display_name("Content Filter")
          .name("content_filter")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(false)
          .description("Filter by content keywords")
          .hint("e.g., urgent, important")
          .build(),
        NodeProperty::builder()
          .display_name("Read Only Unread")
          .name("read_only_unread")
          .kind(hetumind_core::workflow::NodePropertyKind::Boolean)
          .required(false)
          .description("Only process unread emails")
          .value(JsonValue::Bool(true))
          .build(),
        NodeProperty::builder()
          .display_name("Has Attachments")
          .name("has_attachments_filter")
          .kind(hetumind_core::workflow::NodePropertyKind::Boolean)
          .required(false)
          .description("Filter emails with attachments")
          .build(),
        // Attachment Options Section
        NodeProperty::builder()
          .display_name("Download Attachments")
          .name("download_attachments")
          .kind(hetumind_core::workflow::NodePropertyKind::Boolean)
          .required(false)
          .description("Download email attachments")
          .value(JsonValue::Bool(true))
          .build(),
        NodeProperty::builder()
        .display_name("Max Attachment Size (bytes)")
        .name("max_attachment_size")
        .kind(hetumind_core::workflow::NodePropertyKind::Number)
        .required(false)
        .description("Maximum attachment size in bytes")
        .value(JsonValue::Number(serde_json::Number::from(10485760))) // 10MB
        .build(),
        NodeProperty::builder()
          .display_name("Allowed File Types")
          .name("allowed_file_types")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(false)
          .description("Comma-separated list of allowed file extensions")
          .hint("e.g., pdf,doc,jpg,png")
          .build(),
        NodeProperty::builder()
          .display_name("Forbidden File Types")
          .name("forbidden_file_types")
          .kind(hetumind_core::workflow::NodePropertyKind::String)
          .required(false)
          .description("Comma-separated list of forbidden file extensions")
          .hint("e.g., exe,bat,scr")
          .build(),
        // Trigger Settings
        NodeProperty::builder()
          .display_name("Poll Interval (seconds)")
          .name("poll_interval")
          .kind(hetumind_core::workflow::NodePropertyKind::Number)
          .required(true)
          .description("How often to check for new emails")
          .value(JsonValue::Number(serde_json::Number::from(60)))
          .build(),
        NodeProperty::builder()
          .display_name("Enabled")
          .name("enabled")
          .kind(hetumind_core::workflow::NodePropertyKind::Boolean)
          .required(false)
          .description("Enable or disable the trigger")
          .value(JsonValue::Bool(true))
          .build(),
      ]);
    base
  }
}
