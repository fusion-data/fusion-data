//! Send Email 邮件发送节点实现
//!
//! 基于 n8n 的 Send Email 节点设计，使用 lettre 库实现 SMTP 邮件发送功能。
//! 支持多种邮件格式、附件处理、多收件人管理等功能。
//!
//! # 主要功能特性
//! - **多种邮件格式**: 支持纯文本、HTML 或混合格式邮件内容
//! - **灵活收件人管理**: 支持主收件人、抄送（CC）、密送（BCC）
//! - **完整附件支持**: 支持多种附件格式和自动编码
//! - **SMTP 配置**: 完整的 SMTP 服务器配置和认证支持
//! - **邮件归属设置**: 支持回复地址设置和邮件优先级
//! - **错误恢复**: 支持 continue_on_fail 模式的优雅错误处理
//! - **批量发送**: 支持同时发送给多个收件人

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeDefinition, FlowNodeRef, NodeGroupKind, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};

mod send_email_v1;
mod utils;

use send_email_v1::SendEmailV1;

use crate::constants::SEND_EMAIL_NODE_KIND;

/// SMTP 安全连接类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SmtpSecurity {
  /// SSL/TLS 加密连接 (通常使用端口 465/587)
  Ssl,
  /// STARTTLS 加密连接 (通常使用端口 587)
  Starttls,
  /// 无加密连接 (不推荐用于生产环境)
  None,
}

/// 邮件格式类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailFormat {
  /// 纯文本格式
  Text,
  /// HTML 格式
  Html,
  /// 混合格式 (同时包含文本和HTML)
  Both,
}

/// 邮件优先级
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmailPriority {
  /// 高优先级
  High,
  /// 普通优先级
  Normal,
  /// 低优先级
  Low,
}

/// SMTP 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
  /// SMTP 服务器地址
  pub host: String,
  /// SMTP 端口
  pub port: u16,
  /// 安全连接类型
  pub security: SmtpSecurity,
  /// 用户名
  pub username: String,
  /// 密码
  pub password: String,
  /// 发件人邮箱地址
  pub from_email: String,
  /// 发件人显示名称 (可选)
  pub from_name: Option<String>,
  /// 连接超时时间 (秒)
  pub connection_timeout: Option<u64>,
  /// 是否允许未授权证书 (仅用于测试)
  pub allow_unauthorized_certs: Option<bool>,
}

impl SmtpConfig {
  /// 验证 SMTP 配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    if self.host.trim().is_empty() {
      return Err("SMTP host cannot be empty".to_string());
    }

    if self.port == 0 {
      return Err("SMTP port must be greater than 0".to_string());
    }

    if self.username.trim().is_empty() {
      return Err("Username cannot be empty".to_string());
    }

    if self.password.trim().is_empty() {
      return Err("Password cannot be empty".to_string());
    }

    if self.from_email.trim().is_empty() {
      return Err("From email cannot be empty".to_string());
    }

    // 简单的邮箱格式验证
    if !self.from_email.contains('@') || !self.from_email.contains('.') {
      return Err("Invalid from email format".to_string());
    }

    Ok(())
  }

  /// 获取完整的发件人地址
  pub fn get_from_address(&self) -> String {
    match &self.from_name {
      Some(name) => format!("{} <{}>", name, self.from_email),
      None => self.from_email.clone(),
    }
  }
}

/// 附件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentConfig {
  /// 附件字段名称 (来自输入数据的二进制字段)
  pub field_name: String,
  /// 自定义文件名 (可选，如果不使用原始文件名)
  pub custom_filename: Option<String>,
  /// MIME 类型 (可选，自动检测)
  pub content_type: Option<String>,
}

/// Send Email 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailConfig {
  /// SMTP 连接配置
  pub smtp_config: SmtpConfig,
  /// 邮件格式
  pub email_format: EmailFormat,
  /// 邮件主题
  pub subject: String,
  /// 收件人邮箱地址 (支持多个，用逗号分隔)
  pub to_emails: String,
  /// 抄送邮箱地址 (可选)
  pub cc_emails: Option<String>,
  /// 密送邮箱地址 (可选)
  pub bcc_emails: Option<String>,
  /// 回复地址 (可选)
  pub reply_to: Option<String>,
  /// 邮件优先级
  pub priority: Option<EmailPriority>,
  /// 附件配置列表
  pub attachments: Option<Vec<AttachmentConfig>>,
  /// 纯文本内容 (当 email_format 为 Text 或 Both 时使用)
  pub text_content: Option<String>,
  /// HTML 内容 (当 email_format 为 Html 或 Both 时使用)
  pub html_content: Option<String>,
  /// 是否继续执行 (发送失败时)
  pub continue_on_fail: Option<bool>,
  /// 是否添加邮件归属标识
  pub append_attribution: Option<bool>,
  /// 自定义邮件头
  pub custom_headers: Option<std::collections::HashMap<String, String>>,
}

impl SendEmailConfig {
  /// 验证邮件配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    // 验证 SMTP 配置
    self.smtp_config.validate()?;

    // 验证主题
    if self.subject.trim().is_empty() {
      return Err("Subject cannot be empty".to_string());
    }

    // 验证收件人
    if self.to_emails.trim().is_empty() {
      return Err("To emails cannot be empty".to_string());
    }

    // 验证邮件内容
    match self.email_format {
      EmailFormat::Text => {
        if self.text_content.is_none() || self.text_content.as_ref().unwrap().trim().is_empty() {
          return Err("Text content is required for text format".to_string());
        }
      }
      EmailFormat::Html => {
        if self.html_content.is_none() || self.html_content.as_ref().unwrap().trim().is_empty() {
          return Err("HTML content is required for HTML format".to_string());
        }
      }
      EmailFormat::Both => {
        if self.text_content.is_none() || self.text_content.as_ref().unwrap().trim().is_empty() {
          return Err("Text content is required for both format".to_string());
        }
        if self.html_content.is_none() || self.html_content.as_ref().unwrap().trim().is_empty() {
          return Err("HTML content is required for both format".to_string());
        }
      }
    }

    // 验证邮箱地址格式
    for email in self.to_emails.split(',') {
      let email = email.trim();
      if !email.is_empty() && (!email.contains('@') || !email.contains('.')) {
        return Err(format!("Invalid email format: {}", email));
      }
    }

    Ok(())
  }

  /// 解析收件人邮箱地址列表
  pub fn parse_to_emails(&self) -> Vec<String> {
    self
      .to_emails
      .split(',')
      .map(|email| email.trim().to_string())
      .filter(|email| !email.is_empty())
      .collect()
  }

  /// 解析抄送邮箱地址列表
  pub fn parse_cc_emails(&self) -> Vec<String> {
    match &self.cc_emails {
      Some(cc) => cc.split(',').map(|email| email.trim().to_string()).filter(|email| !email.is_empty()).collect(),
      None => Vec::new(),
    }
  }

  /// 解析密送邮箱地址列表
  pub fn parse_bcc_emails(&self) -> Vec<String> {
    match &self.bcc_emails {
      Some(bcc) => bcc.split(',').map(|email| email.trim().to_string()).filter(|email| !email.is_empty()).collect(),
      None => Vec::new(),
    }
  }
}

/// Send Email 节点实现
pub struct SendEmailNode {
  default_version: Version,
  executors: Vec<FlowNodeRef>,
}

impl SendEmailNode {
  /// 创建新的 SendEmail 节点
  pub fn new() -> Result<Self, RegistrationError> {
    let base = Self::base();
    let executors: Vec<FlowNodeRef> = vec![Arc::new(SendEmailV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }

  fn base() -> NodeDefinition {
    NodeDefinition::new(SEND_EMAIL_NODE_KIND, "Send Email")
      .add_group(NodeGroupKind::Output)
      .with_description("Sends emails via SMTP. Supports text/HTML formats, attachments, and multiple recipients.")
      .with_icon("mail")
  }
}

impl Node for SendEmailNode {
  fn default_version(&self) -> &Version {
    &self.default_version
  }

  fn node_executors(&self) -> &[FlowNodeRef] {
    &self.executors
  }

  fn kind(&self) -> NodeKind {
    self.executors[0].definition().kind.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_smtp_config_validation() {
    // 有效的配置
    let valid_config = SmtpConfig {
      host: "smtp.gmail.com".to_string(),
      port: 587,
      security: SmtpSecurity::Starttls,
      username: "test@gmail.com".to_string(),
      password: "password".to_string(),
      from_email: "sender@gmail.com".to_string(),
      from_name: Some("Sender Name".to_string()),
      connection_timeout: Some(30),
      allow_unauthorized_certs: Some(false),
    };
    assert!(valid_config.validate().is_ok());

    // 无效的配置 (空主机名)
    let invalid_config = SmtpConfig {
      host: "".to_string(),
      port: 587,
      security: SmtpSecurity::Starttls,
      username: "test@gmail.com".to_string(),
      password: "password".to_string(),
      from_email: "sender@gmail.com".to_string(),
      from_name: None,
      connection_timeout: None,
      allow_unauthorized_certs: None,
    };
    assert!(invalid_config.validate().is_err());
  }

  #[test]
  fn test_email_config_validation() {
    let smtp_config = SmtpConfig {
      host: "smtp.gmail.com".to_string(),
      port: 587,
      security: SmtpSecurity::Starttls,
      username: "test@gmail.com".to_string(),
      password: "password".to_string(),
      from_email: "sender@gmail.com".to_string(),
      from_name: None,
      connection_timeout: None,
      allow_unauthorized_certs: None,
    };

    // 有效的文本邮件配置
    let valid_text_config = SendEmailConfig {
      smtp_config: smtp_config.clone(),
      email_format: EmailFormat::Text,
      subject: "Test Subject".to_string(),
      to_emails: "recipient@example.com".to_string(),
      cc_emails: None,
      bcc_emails: None,
      reply_to: None,
      priority: None,
      attachments: None,
      text_content: Some("This is a test email.".to_string()),
      html_content: None,
      continue_on_fail: Some(true),
      append_attribution: Some(false),
      custom_headers: None,
    };
    assert!(valid_text_config.validate().is_ok());

    // 无效的配置 (空主题)
    let invalid_config = SendEmailConfig {
      smtp_config,
      email_format: EmailFormat::Text,
      subject: "".to_string(),
      to_emails: "recipient@example.com".to_string(),
      cc_emails: None,
      bcc_emails: None,
      reply_to: None,
      priority: None,
      attachments: None,
      text_content: Some("Content".to_string()),
      html_content: None,
      continue_on_fail: None,
      append_attribution: None,
      custom_headers: None,
    };
    assert!(invalid_config.validate().is_err());
  }

  #[test]
  fn test_email_parsing() {
    let config = SendEmailConfig {
      smtp_config: SmtpConfig {
        host: "smtp.gmail.com".to_string(),
        port: 587,
        security: SmtpSecurity::Starttls,
        username: "test@gmail.com".to_string(),
        password: "password".to_string(),
        from_email: "sender@gmail.com".to_string(),
        from_name: None,
        connection_timeout: None,
        allow_unauthorized_certs: None,
      },
      email_format: EmailFormat::Text,
      subject: "Test".to_string(),
      to_emails: "user1@example.com, user2@example.com , user3@example.com".to_string(),
      cc_emails: Some("cc1@example.com, cc2@example.com".to_string()),
      bcc_emails: None,
      reply_to: None,
      priority: None,
      attachments: None,
      text_content: Some("Test".to_string()),
      html_content: None,
      continue_on_fail: None,
      append_attribution: None,
      custom_headers: None,
    };

    let to_emails = config.parse_to_emails();
    assert_eq!(to_emails.len(), 3);
    assert!(to_emails.contains(&"user1@example.com".to_string()));
    assert!(to_emails.contains(&"user2@example.com".to_string()));
    assert!(to_emails.contains(&"user3@example.com".to_string()));

    let cc_emails = config.parse_cc_emails();
    assert_eq!(cc_emails.len(), 2);
    assert!(cc_emails.contains(&"cc1@example.com".to_string()));
    assert!(cc_emails.contains(&"cc2@example.com".to_string()));
  }

  #[test]
  fn test_serialization() {
    // 测试枚举序列化
    let format = EmailFormat::Html;
    let serialized = serde_json::to_string(&format).unwrap();
    let deserialized: EmailFormat = serde_json::from_str(&serialized).unwrap();
    assert_eq!(format, deserialized);

    let security = SmtpSecurity::Ssl;
    let serialized = serde_json::to_string(&security).unwrap();
    let deserialized: SmtpSecurity = serde_json::from_str(&serialized).unwrap();
    assert_eq!(security, deserialized);

    let priority = EmailPriority::High;
    let serialized = serde_json::to_string(&priority).unwrap();
    let deserialized: EmailPriority = serde_json::from_str(&serialized).unwrap();
    assert_eq!(priority, deserialized);
  }

  #[test]
  fn test_node_creation() {
    let node = SendEmailNode::new();
    assert!(node.is_ok());

    let node = node.unwrap();
    assert_eq!(node.default_version().major, 1);
    assert_eq!(node.node_executors().len(), 1);
  }

  #[test]
  fn test_from_address_formatting() {
    let config_with_name = SmtpConfig {
      host: "smtp.gmail.com".to_string(),
      port: 587,
      security: SmtpSecurity::Starttls,
      username: "test@gmail.com".to_string(),
      password: "password".to_string(),
      from_email: "sender@gmail.com".to_string(),
      from_name: Some("John Doe".to_string()),
      connection_timeout: None,
      allow_unauthorized_certs: None,
    };

    assert_eq!(config_with_name.get_from_address(), "John Doe <sender@gmail.com>");

    let config_without_name = SmtpConfig {
      host: "smtp.gmail.com".to_string(),
      port: 587,
      security: SmtpSecurity::Starttls,
      username: "test@gmail.com".to_string(),
      password: "password".to_string(),
      from_email: "sender@gmail.com".to_string(),
      from_name: None,
      connection_timeout: None,
      allow_unauthorized_certs: None,
    };

    assert_eq!(config_without_name.get_from_address(), "sender@gmail.com");
  }
}
