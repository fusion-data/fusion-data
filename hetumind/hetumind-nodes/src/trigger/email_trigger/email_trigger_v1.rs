use std::sync::Arc;

use async_trait::async_trait;
use hetumind_core::types::JsonValue;
use hetumind_core::workflow::{
  ConnectionKind, ExecutionDataItems, ExecutionDataMap, FlowNode, NodeDefinition, NodeExecutionContext,
  NodeExecutionError, NodeGroupKind, NodeProperty, NodePropertyKind, RegistrationError, make_execution_data_map,
};

use crate::constants::EMAIL_TRIGGER_NODE_KIND;

use serde_json::json;

use super::{EmailProcessingConfig, EmailReadFormat, EmailTriggerConfig, ImapAuthentication, ImapSecurity};

pub struct EmailTriggerV1 {
  definition: Arc<NodeDefinition>,
}

impl TryFrom<NodeDefinition> for EmailTriggerV1 {
  type Error = RegistrationError;

  fn try_from(definition: NodeDefinition) -> Result<Self, Self::Error> {
    Ok(Self { definition: Arc::new(definition) })
  }
}

#[async_trait]
impl FlowNode for EmailTriggerV1 {
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
    let connection_timeout = parameters.get("connection_timeout").and_then(|v| v.as_u64());
    let read_timeout = parameters.get("read_timeout").and_then(|v| v.as_u64());

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

    let max_attachment_size = parameters.get("max_attachment_size").and_then(|v| v.as_u64());

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
    parameters.get("poll_interval").and_then(|v| v.as_u64()).ok_or_else(|| {
      NodeExecutionError::ParameterValidation(hetumind_core::workflow::ValidationError::RequiredFieldMissing {
        field: "poll_interval".to_string(),
      })
    })
  }

  /// 解析启用状态
  fn parse_enabled(&self, parameters: &JsonValue) -> Result<bool, NodeExecutionError> {
    Ok(parameters.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true))
  }
}

impl EmailTriggerV1 {
  /// 创建基础的节点定义
  pub fn create_base() -> NodeDefinition {
    NodeDefinition::new(EMAIL_TRIGGER_NODE_KIND, "Email Trigger")
      .add_group(NodeGroupKind::Trigger)
      .with_description("Triggers workflow when new emails are received via IMAP")
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("IMAP Host")
          .with_name("imap_host")
          .with_required(true)
          .with_description("IMAP server hostname (e.g., imap.gmail.com)")
          .with_hint("e.g., imap.gmail.com"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("IMAP Port")
          .with_name("imap_port")
          .with_required(true)
          .with_description("IMAP server port number")
          .with_value(json!(993)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("Security")
          .with_name("imap_security")
          .with_options(vec![
            Box::new(NodeProperty::new_option("SSL/TLS", "ssl", json!("ssl"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("STARTTLS", "starttls", json!("starttls"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("None", "none", json!("none"), NodePropertyKind::String)),
          ])
          .with_required(true)
          .with_description("Connection security mode")
          .with_value(json!("ssl")),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("Authentication")
          .with_name("imap_authentication")
          .with_options(vec![
            Box::new(NodeProperty::new_option("Normal", "normal", json!("normal"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("OAuth 2.0", "oauth", json!("oauth"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("CRAM-MD5", "cram_md5", json!("cram_md5"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option(
              "Digest MD5",
              "digest_md5",
              json!("digest_md5"),
              NodePropertyKind::String,
            )),
          ])
          .with_required(true)
          .with_description("IMAP authentication method")
          .with_value(json!("normal")),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("Username")
          .with_name("imap_username")
          .with_required(true)
          .with_description("IMAP username")
          .with_hint("e.g., your-email@gmail.com"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("Password")
          .with_name("imap_password")
          .with_required(true)
          .with_description("IMAP password or access token")
          .with_hint("Use app-specific password for Gmail"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("Mailbox")
          .with_name("imap_mailbox")
          .with_required(false)
          .with_description("Mailbox name (default: INBOX)")
          .with_value(json!("INBOX")),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("Connection Timeout (seconds)")
          .with_name("connection_timeout")
          .with_required(false)
          .with_description("Connection timeout in seconds")
          .with_value(json!(30)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("Read Timeout (seconds)")
          .with_name("read_timeout")
          .with_required(false)
          .with_description("Read timeout in seconds")
          .with_value(json!(60)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Options)
          .with_display_name("Email Read Format")
          .with_name("email_read_format")
          .with_options(vec![
            Box::new(NodeProperty::new_option("Raw", "raw", json!("raw"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("Resolved", "resolved", json!("resolved"), NodePropertyKind::String)),
            Box::new(NodeProperty::new_option("Simple", "simple", json!("simple"), NodePropertyKind::String)),
          ])
          .with_required(true)
          .with_description("Email output format")
          .with_value(json!("resolved")),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("Max Emails per Poll")
          .with_name("max_emails")
          .with_required(false)
          .with_description("Maximum number of emails to process per poll")
          .with_value(json!(50)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Mark as Read")
          .with_name("mark_as_read")
          .with_required(false)
          .with_description("Mark processed emails as read")
          .with_value(json!(true)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Delete After Read")
          .with_name("delete_after_read")
          .with_required(false)
          .with_description("Delete emails after processing")
          .with_value(json!(false)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Enable Filter")
          .with_name("enable_filter")
          .with_required(false)
          .with_description("Enable email filtering")
          .with_value(json!(false)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("From Filter")
          .with_name("from_filter")
          .with_required(false)
          .with_description("Filter by sender (supports wildcards)")
          .with_hint("e.g., noreply@*.com, support@company.com"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("To Filter")
          .with_name("to_filter")
          .with_required(false)
          .with_description("Filter by recipient (supports wildcards)")
          .with_hint("e.g., support@*.com"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("Subject Filter")
          .with_name("subject_filter")
          .with_required(false)
          .with_description("Filter by subject (supports wildcards)")
          .with_hint("e.g., *Invoice*, *Alert*"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("Content Filter")
          .with_name("content_filter")
          .with_required(false)
          .with_description("Filter by content keywords")
          .with_hint("e.g., urgent, important"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Read Only Unread")
          .with_name("read_only_unread")
          .with_required(false)
          .with_description("Only process unread emails")
          .with_value(json!(true)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Has Attachments")
          .with_name("has_attachments_filter")
          .with_required(false)
          .with_description("Filter emails with attachments"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Download Attachments")
          .with_name("download_attachments")
          .with_required(false)
          .with_description("Download email attachments")
          .with_value(json!(true)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("Max Attachment Size (bytes)")
          .with_name("max_attachment_size")
          .with_required(false)
          .with_description("Maximum attachment size in bytes")
          .with_value(json!(10485760)), // 10MB
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("Allowed File Types")
          .with_name("allowed_file_types")
          .with_required(false)
          .with_description("Comma-separated list of allowed file extensions")
          .with_hint("e.g., pdf,doc,jpg,png"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::String)
          .with_display_name("Forbidden File Types")
          .with_name("forbidden_file_types")
          .with_required(false)
          .with_description("Comma-separated list of forbidden file extensions")
          .with_hint("e.g., exe,bat,scr"),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Number)
          .with_display_name("Poll Interval (seconds)")
          .with_name("poll_interval")
          .with_required(true)
          .with_description("How often to check for new emails")
          .with_value(json!(60)),
      )
      .add_property(
        NodeProperty::new(NodePropertyKind::Boolean)
          .with_display_name("Enabled")
          .with_name("enabled")
          .with_required(false)
          .with_description("Enable or disable the trigger")
          .with_value(json!(true)),
      )
  }
}
