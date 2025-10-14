//! EmailTriggerNode 邮件触发器节点实现
//!
//! 基于 n8n Email Trigger (IMAP) v2.1 设计，用于监听邮箱并触发工作流
//!
//! # 主要功能特性
//! - **IMAP 邮件监听**: 支持标准 IMAP 协议连接邮箱
//! - **实时邮件监控**: 实时监控新邮件并触发工作流
//! - **多种邮件格式**: 支持 RAW、Resolved、Simple 三种输出格式
//! - **附件处理**: 完整支持邮件附件的二进制数据处理
//! - **邮件过滤**: 支持发件人、主题、内容等过滤条件
//! - **重复防护**: 基于 UID 的邮件重复处理防护
//! - **连接管理**: 自动重连和连接池管理
//! - **安全认证**: 支持多种 IMAP 认证方式

use std::sync::Arc;

use hetumind_core::{
  version::Version,
  workflow::{Node, NodeExecutor, NodeKind, RegistrationError},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod email_trigger_v1;
mod utils;

use email_trigger_v1::EmailTriggerV1;

/// 邮件读取格式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailReadFormat {
  /// 原始邮件格式（包含完整的 MIME 结构）
  Raw,
  /// 解析后的邮件格式（结构化的邮件数据）
  Resolved,
  /// 简化邮件格式（只包含核心字段）
  Simple,
}

/// IMAP 认证方式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImapAuthentication {
  /// 常规用户名密码认证
  Normal,
  /// OAuth 2.0 认证
  OAuth,
  /// CRAM-MD5 认证
  CramMd5,
  /// Digest MD5 认证
  DigestMd5,
}

/// SSL/TLS 连接安全模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImapSecurity {
  /// SSL 加密连接
  Ssl,
  /// STARTTLS 加密连接
  Starttls,
  /// 无加密连接
  None,
}

/// 邮件过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailFilter {
  /// 发件人过滤（支持通配符）
  pub from_filter: Option<String>,
  /// 收件人过滤（支持通配符）
  pub to_filter: Option<String>,
  /// 主题过滤（支持通配符）
  pub subject_filter: Option<String>,
  /// 内容关键词过滤
  pub content_filter: Option<String>,
  /// 是否只读取未读邮件
  pub read_only_unread: Option<bool>,
  /// 是否有附件
  pub has_attachments: Option<bool>,
}

/// 附件处理选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentOptions {
  /// 是否下载附件
  pub download_attachments: Option<bool>,
  /// 附件大小限制（字节）
  pub max_attachment_size: Option<u64>,
  /// 允许的附件文件类型
  pub allowed_file_types: Option<Vec<String>>,
  /// 禁止的附件文件类型
  pub forbidden_file_types: Option<Vec<String>>,
}

/// IMAP 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImapConnectionConfig {
  /// IMAP 服务器地址
  pub host: String,
  /// IMAP 端口
  pub port: u16,
  /// 安全连接模式
  pub security: ImapSecurity,
  /// 认证方式
  pub authentication: ImapAuthentication,
  /// 用户名
  pub username: String,
  /// 密码或访问令牌
  pub password: String,
  /// 邮箱名称（默认 INBOX）
  pub mailbox: Option<String>,
  /// 连接超时时间（秒）
  pub connection_timeout: Option<u64>,
  /// 读取超时时间（秒）
  pub read_timeout: Option<u64>,
}

/// 邮件处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailProcessingConfig {
  /// 邮件读取格式
  pub read_format: EmailReadFormat,
  /// 邮件过滤条件
  pub filter: Option<EmailFilter>,
  /// 附件处理选项
  pub attachment_options: Option<AttachmentOptions>,
  /// 邮件数量限制（每次触发读取的最大邮件数）
  pub max_emails: Option<u32>,
  /// 是否标记为已读
  pub mark_as_read: Option<bool>,
  /// 是否删除处理后的邮件
  pub delete_after_read: Option<bool>,
}

/// Email Trigger 节点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTriggerConfig {
  /// IMAP 连接配置
  pub imap_connection: ImapConnectionConfig,
  /// 邮件处理配置
  pub processing: EmailProcessingConfig,
  /// 轮询间隔（秒）
  pub poll_interval: u64,
  /// 是否启用触发器
  pub enabled: bool,
}

impl EmailTriggerConfig {
  /// 验证配置是否有效
  pub fn validate(&self) -> Result<(), String> {
    // 验证 IMAP 连接配置
    if self.imap_connection.host.trim().is_empty() {
      return Err("IMAP host cannot be empty".to_string());
    }

    if self.imap_connection.port == 0 {
      return Err("IMAP port must be greater than 0".to_string());
    }

    if self.imap_connection.username.trim().is_empty() {
      return Err("Username cannot be empty".to_string());
    }

    if self.imap_connection.password.trim().is_empty() {
      return Err("Password cannot be empty".to_string());
    }

    // 验证轮询间隔
    if self.poll_interval == 0 {
      return Err("Poll interval must be greater than 0".to_string());
    }

    // 验证邮件数量限制
    if let Some(max_emails) = self.processing.max_emails {
      if max_emails == 0 {
        return Err("Max emails must be greater than 0".to_string());
      }
    }

    // 验证附件大小限制
    if let Some(attachment_options) = &self.processing.attachment_options {
      if let Some(max_size) = attachment_options.max_attachment_size {
        if max_size == 0 {
          return Err("Max attachment size must be greater than 0".to_string());
        }
      }
    }

    Ok(())
  }

  /// 生成触发器标识符
  pub fn generate_trigger_id(&self) -> String {
    format!(
      "email_trigger_{}_{}_{}",
      self.imap_connection.host.replace('.', "_"),
      self.imap_connection.port,
      self.imap_connection.username.replace('@', "_")
    )
  }

  /// 获取默认邮箱名称
  pub fn get_mailbox(&self) -> String {
    self.imap_connection.mailbox.clone().unwrap_or_else(|| "INBOX".to_string())
  }
}

/// 邮件数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailData {
  /// 邮件唯一标识符
  pub uid: u32,
  /// 消息 ID
  pub message_id: Option<String>,
  /// 发件人
  pub from: Option<String>,
  /// 收件人列表
  pub to: Vec<String>,
  /// 抄送列表
  pub cc: Vec<String>,
  /// 密送列表
  pub bcc: Vec<String>,
  /// 主题
  pub subject: Option<String>,
  /// 邮件正文（纯文本）
  pub text_body: Option<String>,
  /// 邮件正文（HTML）
  pub html_body: Option<String>,
  /// 发送时间
  pub sent_date: Option<String>,
  /// 接收时间
  pub received_date: Option<String>,
  /// 附件列表
  pub attachments: Vec<EmailAttachment>,
  /// 原始邮件大小（字节）
  pub size: Option<u64>,
  /// 是否已读
  pub is_read: bool,
  /// 是否包含附件
  pub has_attachments: bool,
  /// 自定义属性
  pub custom_properties: Option<Value>,
}

/// 邮件附件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
  /// 附件文件名
  pub filename: String,
  /// MIME 类型
  pub content_type: String,
  /// 文件大小（字节）
  pub size: u64,
  /// 附件内容（Base64 编码）
  pub data: Option<String>,
  /// 内容 ID（用于内联附件）
  pub content_id: Option<String>,
  /// 附件在邮件中的位置
  pub part_number: Option<String>,
}

/// Email Trigger 节点实现
pub struct EmailTriggerNode {
  default_version: Version,
  executors: Vec<NodeExecutor>,
}

impl EmailTriggerNode {
  /// 创建新的 EmailTrigger 节点
  pub fn new() -> Result<Self, RegistrationError> {
    let base = EmailTriggerV1::create_base();
    let executors: Vec<NodeExecutor> = vec![Arc::new(EmailTriggerV1::try_from(base)?)];
    let default_version = executors.iter().map(|node| node.definition().version.clone()).max().unwrap();
    Ok(Self { default_version, executors })
  }
}

impl Node for EmailTriggerNode {
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
  use serde_json::json;

  use super::*;

  #[test]
  fn test_email_trigger_config_validation() {
    // 有效的配置
    let valid_config = EmailTriggerConfig {
      imap_connection: ImapConnectionConfig {
        host: "imap.gmail.com".to_string(),
        port: 993,
        security: ImapSecurity::Ssl,
        authentication: ImapAuthentication::Normal,
        username: "test@gmail.com".to_string(),
        password: "password".to_string(),
        mailbox: Some("INBOX".to_string()),
        connection_timeout: Some(30),
        read_timeout: Some(60),
      },
      processing: EmailProcessingConfig {
        read_format: EmailReadFormat::Resolved,
        filter: Some(EmailFilter {
          from_filter: Some("noreply@*.com".to_string()),
          to_filter: None,
          subject_filter: None,
          content_filter: None,
          read_only_unread: Some(true),
          has_attachments: None,
        }),
        attachment_options: Some(AttachmentOptions {
          download_attachments: Some(true),
          max_attachment_size: Some(10 * 1024 * 1024), // 10MB
          allowed_file_types: Some(vec!["pdf".to_string(), "jpg".to_string()]),
          forbidden_file_types: Some(vec!["exe".to_string()]),
        }),
        max_emails: Some(50),
        mark_as_read: Some(true),
        delete_after_read: Some(false),
      },
      poll_interval: 60,
      enabled: true,
    };
    assert!(valid_config.validate().is_ok());

    // 无效的配置（空主机名）
    let invalid_config = EmailTriggerConfig {
      imap_connection: ImapConnectionConfig {
        host: "".to_string(),
        port: 993,
        security: ImapSecurity::Ssl,
        authentication: ImapAuthentication::Normal,
        username: "test@gmail.com".to_string(),
        password: "password".to_string(),
        mailbox: None,
        connection_timeout: None,
        read_timeout: None,
      },
      processing: EmailProcessingConfig {
        read_format: EmailReadFormat::Simple,
        filter: None,
        attachment_options: None,
        max_emails: None,
        mark_as_read: None,
        delete_after_read: None,
      },
      poll_interval: 60,
      enabled: true,
    };
    assert!(invalid_config.validate().is_err());
  }

  #[test]
  fn test_serialization() {
    // 测试枚举序列化
    let format = EmailReadFormat::Resolved;
    let serialized = serde_json::to_string(&format).unwrap();
    let deserialized: EmailReadFormat = serde_json::from_str(&serialized).unwrap();
    assert_eq!(format, deserialized);

    let auth = ImapAuthentication::OAuth;
    let serialized = serde_json::to_string(&auth).unwrap();
    let deserialized: ImapAuthentication = serde_json::from_str(&serialized).unwrap();
    assert_eq!(auth, deserialized);

    let security = ImapSecurity::Starttls;
    let serialized = serde_json::to_string(&security).unwrap();
    let deserialized: ImapSecurity = serde_json::from_str(&serialized).unwrap();
    assert_eq!(security, deserialized);
  }

  #[test]
  fn test_trigger_id_generation() {
    let config = EmailTriggerConfig {
      imap_connection: ImapConnectionConfig {
        host: "imap.gmail.com".to_string(),
        port: 993,
        security: ImapSecurity::Ssl,
        authentication: ImapAuthentication::Normal,
        username: "test@gmail.com".to_string(),
        password: "password".to_string(),
        mailbox: None,
        connection_timeout: None,
        read_timeout: None,
      },
      processing: EmailProcessingConfig {
        read_format: EmailReadFormat::Simple,
        filter: None,
        attachment_options: None,
        max_emails: None,
        mark_as_read: None,
        delete_after_read: None,
      },
      poll_interval: 60,
      enabled: true,
    };

    let trigger_id = config.generate_trigger_id();
    assert!(trigger_id.starts_with("email_trigger_"));
    assert!(!trigger_id.contains('.'));
    assert!(!trigger_id.contains('@'));
  }

  #[test]
  fn test_node_creation() {
    let node = EmailTriggerNode::new();
    assert!(node.is_ok());

    let node = node.unwrap();
    assert_eq!(node.default_version().major, 1);
    assert_eq!(node.node_executors().len(), 1);
  }
}
