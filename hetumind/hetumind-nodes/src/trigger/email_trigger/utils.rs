//! Email Trigger 工具函数实现
//!
//! 提供邮件处理、IMAP 连接管理、邮件解析等核心功能

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose};
use regex::Regex;
use serde_json::{Value, json};

use super::{EmailAttachment, EmailData, EmailFilter, EmailReadFormat, EmailTriggerConfig};

/// IMAP 连接器
pub struct ImapConnector {
  config: super::ImapConnectionConfig,
  /// 连接状态
  connected: bool,
  /// 最后检查时间戳
  last_check_timestamp: Option<u64>,
  /// 已处理的邮件 UID 集合
  processed_uids: std::collections::HashSet<u32>,
}

impl ImapConnector {
  /// 创建新的 IMAP 连接器
  pub fn new(config: super::ImapConnectionConfig) -> Self {
    Self { config, connected: false, last_check_timestamp: None, processed_uids: std::collections::HashSet::new() }
  }

  /// 连接到 IMAP 服务器
  pub async fn connect(&mut self) -> Result<(), String> {
    // 这里应该实现实际的 IMAP 连接逻辑
    // 由于这是一个示例实现，我们只模拟连接过程
    self.connected = true;
    self.last_check_timestamp = Some(
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_secs(),
    );
    Ok(())
  }

  /// 检查连接状态
  pub fn is_connected(&self) -> bool {
    self.connected
  }

  /// 断开连接
  pub fn disconnect(&mut self) {
    self.connected = false;
    self.last_check_timestamp = None;
  }

  /// 获取新邮件列表
  pub async fn fetch_new_emails(&mut self, config: &EmailTriggerConfig) -> Result<Vec<EmailData>, String> {
    if !self.connected {
      return Err("Not connected to IMAP server".to_string());
    }

    // 这里应该实现实际的 IMAP 邮件获取逻辑
    // 由于这是一个示例实现，我们返回模拟数据
    let mock_emails = self.create_mock_emails(config).await?;

    // 过滤已处理的邮件
    let new_emails: Vec<EmailData> =
      mock_emails.into_iter().filter(|email| !self.processed_uids.contains(&email.uid)).collect();

    // 标记邮件为已处理
    for email in &new_emails {
      self.processed_uids.insert(email.uid);
    }

    Ok(new_emails)
  }

  /// 创建模拟邮件数据（用于测试）
  async fn create_mock_emails(&self, config: &EmailTriggerConfig) -> Result<Vec<EmailData>, String> {
    let mut emails = Vec::new();

    // 创建模拟邮件
    let email1 = EmailData {
      uid: 1001,
      message_id: Some("<message1@example.com>".to_string()),
      from: Some("sender1@example.com".to_string()),
      to: vec!["recipient@example.com".to_string()],
      cc: vec![],
      bcc: vec![],
      subject: Some("Test Email 1".to_string()),
      text_body: Some("This is a test email with some content.".to_string()),
      html_body: Some("<p>This is a <strong>test</strong> email.</p>".to_string()),
      sent_date: Some("2024-01-15T10:30:00Z".to_string()),
      received_date: Some("2024-01-15T10:31:00Z".to_string()),
      attachments: vec![EmailAttachment {
        filename: "test.pdf".to_string(),
        content_type: "application/pdf".to_string(),
        size: 1024,
        data: Some("JVBERi0xLjQKJeLjz9MKNCAwIG9iago8PAovVHlwZSAvQ2F0YWxvZwovT3V0bGluZXMgMiAwIFIKL1BhZ2VzIDMgMCBSCj4+CmVuZG9iagoyIDAgb2JqCjw8Ci9UeXBlIC9PdXRsaW5lcwovQ291bnQgMAo+PgplbmRvYmoKMyAwIG9iago8PAovVHlwZSAvUGFnZXMKL0NvdW50IDEKL0tpZHMgWzQgMCBSXQo+PgplbmRvYmoKNCAwIG9iago8PAovVHlwZSAvUGFnZQovUGFyZW50IDMgMCBSCi9NZWRpYUJveCBbMCAwIDYxMiA3OTJdCi9Db250ZW50cyA1IDAgUgo+PgplbmRvYmoKNSAwIG9iago8PAovTGVuZ3RoIDQ0Cj4+CnN0cmVhbQpCVAovRjEgMTIgVGYKNzIgNzIwIFRkCihUZXN0IERvY3VtZW50KSBUagpFVApzdHJlYW0KZW5kb2JqCnhyZWYKMCA2CjAwMDAwMDAwMDAgNjU1MzUgZiAKMDAwMDAwMDAxNSAwMDAwMCBuIAowMDAwMDAwMDc0IDAwMDAwIG4gCjAwMDAwMDAxMjAgMDAwMDAgbiAKMDAwMDAwMDE3NyAwMDAwMCBuIAowMDAwMDAwMjU2IDAwMDAwIG4gCnRyYWlsZXIKPDwKL1NpemUgNgovUm9vdCAxIDAgUgo+PgpzdGFydHhyZWYKMzE2CiUlRU9G".to_string()),
        content_id: None,
        part_number: Some("1.1".to_string()),
      }],
      size: Some(2048),
      is_read: false,
      has_attachments: true,
      custom_properties: Some(json!({
        "priority": "high",
        "importance": "normal"
      })),
    };

    let email2 = EmailData {
      uid: 1002,
      message_id: Some("<message2@example.com>".to_string()),
      from: Some("sender2@example.com".to_string()),
      to: vec!["recipient@example.com".to_string()],
      cc: vec!["cc@example.com".to_string()],
      bcc: vec![],
      subject: Some("Urgent Alert".to_string()),
      text_body: Some("This is an urgent alert message.".to_string()),
      html_body: Some(
        "<div style='color: red;'><h1>ALERT</h1><p>This is an urgent alert message.</p></div>".to_string(),
      ),
      sent_date: Some("2024-01-15T11:00:00Z".to_string()),
      received_date: Some("2024-01-15T11:01:00Z".to_string()),
      attachments: vec![],
      size: Some(512),
      is_read: false,
      has_attachments: false,
      custom_properties: Some(json!({
        "priority": "urgent",
        "importance": "high"
      })),
    };

    emails.push(email1);
    emails.push(email2);

    // 应用过滤条件
    if let Some(filter) = &config.processing.filter {
      emails = filter_emails(emails, filter);
    }

    Ok(emails)
  }

  /// 标记邮件为已读
  pub async fn mark_as_read(&mut self, _uid: u32) -> Result<(), String> {
    if !self.connected {
      return Err("Not connected to IMAP server".to_string());
    }

    // 实现实际的邮件标记逻辑
    Ok(())
  }

  /// 删除邮件
  pub async fn delete_email(&mut self, uid: u32) -> Result<(), String> {
    if !self.connected {
      return Err("Not connected to IMAP server".to_string());
    }

    // 实现实际的邮件删除逻辑
    self.processed_uids.remove(&uid);
    Ok(())
  }
}

/// 邮件过滤器
pub fn filter_emails(emails: Vec<EmailData>, filter: &EmailFilter) -> Vec<EmailData> {
  emails
    .into_iter()
    .filter(|email| {
      // 检查发件人过滤
      if let Some(from_filter) = &filter.from_filter {
        if !matches_pattern(&email.from.as_deref().unwrap_or(""), from_filter) {
          return false;
        }
      }

      // 检查收件人过滤
      if let Some(to_filter) = &filter.to_filter {
        let matches_to = email.to.iter().any(|recipient| matches_pattern(recipient, to_filter))
          || email.cc.iter().any(|recipient| matches_pattern(recipient, to_filter))
          || email.bcc.iter().any(|recipient| matches_pattern(recipient, to_filter));

        if !matches_to {
          return false;
        }
      }

      // 检查主题过滤
      if let Some(subject_filter) = &filter.subject_filter {
        if !matches_pattern(&email.subject.as_deref().unwrap_or(""), subject_filter) {
          return false;
        }
      }

      // 检查内容过滤
      if let Some(content_filter) = &filter.content_filter {
        let text_content =
          format!("{} {}", email.text_body.as_deref().unwrap_or(""), email.html_body.as_deref().unwrap_or(""));
        if !text_content.to_lowercase().contains(&content_filter.to_lowercase()) {
          return false;
        }
      }

      // 检查只读未读邮件
      if let Some(read_only_unread) = filter.read_only_unread
        && read_only_unread
        && email.is_read
      {
        return false;
      }

      // 检查是否有附件
      if let Some(has_attachments) = filter.has_attachments {
        if has_attachments != email.has_attachments {
          return false;
        }
      }

      true
    })
    .collect()
}

/// 模式匹配函数（支持通配符）
fn matches_pattern(text: &str, pattern: &str) -> bool {
  // 转换通配符模式为正则表达式
  let regex_pattern = pattern.replace('.', r"\.").replace('*', ".*").replace('?', ".");

  match Regex::new(&format!("(?i)^{}$", regex_pattern)) {
    Ok(re) => re.is_match(text),
    Err(_) => text.to_lowercase().contains(&pattern.to_lowercase()),
  }
}

/// 邮件格式转换器
pub struct EmailFormatter;

impl EmailFormatter {
  /// 根据指定格式转换邮件数据
  pub fn format_email(email: EmailData, format: &EmailReadFormat) -> Result<Value, String> {
    match format {
      EmailReadFormat::Raw => Self::format_raw(email),
      EmailReadFormat::Resolved => Self::format_resolved(email),
      EmailReadFormat::Simple => Self::format_simple(email),
    }
  }

  /// 格式化为原始邮件格式
  fn format_raw(email: EmailData) -> Result<Value, String> {
    let raw_data = json!({
      "uid": email.uid,
      "message_id": email.message_id,
      "raw_data": Self::generate_raw_mime(&email),
      "size": email.size,
      "received_date": email.received_date,
    });
    Ok(raw_data)
  }

  /// 格式化为解析后的邮件格式
  fn format_resolved(email: EmailData) -> Result<Value, String> {
    let resolved_data = json!({
      "uid": email.uid,
      "message_id": email.message_id,
      "from": email.from,
      "to": email.to,
      "cc": email.cc,
      "bcc": email.bcc,
      "subject": email.subject,
      "text_body": email.text_body,
      "html_body": email.html_body,
      "sent_date": email.sent_date,
      "received_date": email.received_date,
      "attachments": email.attachments.into_iter().map(|att| json!({
        "filename": att.filename,
        "content_type": att.content_type,
        "size": att.size,
        "data": att.data,
        "content_id": att.content_id,
        "part_number": att.part_number,
      })).collect::<Vec<_>>(),
      "size": email.size,
      "is_read": email.is_read,
      "has_attachments": email.has_attachments,
      "custom_properties": email.custom_properties,
    });
    Ok(resolved_data)
  }

  /// 格式化为简化邮件格式
  fn format_simple(email: EmailData) -> Result<Value, String> {
    let simple_data = json!({
      "uid": email.uid,
      "from": email.from,
      "to": email.to.first().unwrap_or(&"".to_string()),
      "subject": email.subject,
      "body": email.text_body.as_deref().unwrap_or_else(|| email.html_body.as_deref().unwrap_or("")),
      "received_date": email.received_date,
      "has_attachments": email.has_attachments,
      "attachment_count": email.attachments.len(),
    });
    Ok(simple_data)
  }

  /// 生成模拟的原始 MIME 数据
  fn generate_raw_mime(email: &EmailData) -> String {
    let mut mime = String::new();

    mime.push_str(&format!("Message-ID: {}\r\n", email.message_id.as_deref().unwrap_or("")));
    mime.push_str(&format!("From: {}\r\n", email.from.as_deref().unwrap_or("")));
    mime.push_str(&format!("To: {}\r\n", email.to.join(", ")));
    if !email.cc.is_empty() {
      mime.push_str(&format!("Cc: {}\r\n", email.cc.join(", ")));
    }
    mime.push_str(&format!("Subject: {}\r\n", email.subject.as_deref().unwrap_or("")));
    mime.push_str(&format!("Date: {}\r\n", email.sent_date.as_deref().unwrap_or("")));
    mime.push_str("\r\n");

    if let Some(text_body) = &email.text_body {
      mime.push_str(text_body);
    }

    mime
  }
}

/// 附件处理器
pub struct AttachmentProcessor;

impl AttachmentProcessor {
  /// 处理附件列表
  pub fn process_attachments(
    attachments: Vec<EmailAttachment>,
    options: &super::AttachmentOptions,
  ) -> Result<Vec<EmailAttachment>, String> {
    let mut processed_attachments = Vec::new();

    for attachment in attachments {
      // 检查文件大小限制
      if let Some(max_size) = options.max_attachment_size {
        if attachment.size > max_size {
          continue; // 跳过超过大小限制的附件
        }
      }

      // 检查允许的文件类型
      if let Some(allowed_types) = &options.allowed_file_types {
        let file_ext = Self::get_file_extension(&attachment.filename);
        if !allowed_types.iter().any(|ext| ext.to_lowercase() == file_ext) {
          continue; // 跳过不允许的文件类型
        }
      }

      // 检查禁止的文件类型
      if let Some(forbidden_types) = &options.forbidden_file_types {
        let file_ext = Self::get_file_extension(&attachment.filename);
        if forbidden_types.iter().any(|ext| ext.to_lowercase() == file_ext) {
          continue; // 跳过禁止的文件类型
        }
      }

      processed_attachments.push(attachment);
    }

    Ok(processed_attachments)
  }

  /// 获取文件扩展名
  fn get_file_extension(filename: &str) -> String {
    filename.rsplit('.').next().unwrap_or("").to_lowercase()
  }

  /// 下载附件内容（模拟实现）
  pub async fn download_attachment(&self, attachment: &mut EmailAttachment) -> Result<(), String> {
    // 这里应该实现实际的附件下载逻辑
    // 由于这是一个示例实现，我们使用模拟数据
    if attachment.data.is_none() {
      attachment.data = Some(Self::generate_mock_attachment_data(&attachment.filename));
    }
    Ok(())
  }

  /// 生成模拟附件数据
  fn generate_mock_attachment_data(filename: &str) -> String {
    let content = match Self::get_file_extension(filename).as_str() {
      "txt" => "This is a mock text file content.",
      "json" => "{\"mock\": \"data\", \"filename\": \"test.txt\"}",
      _ => "MOCK_BINARY_DATA_HERE",
    };
    general_purpose::STANDARD.encode(content.as_bytes())
  }
}

/// 邮件触发器状态管理
pub struct TriggerStateManager {
  /// 触发器状态存储
  states: HashMap<String, TriggerState>,
}

impl TriggerStateManager {
  /// 创建新的状态管理器
  pub fn new() -> Self {
    Self { states: HashMap::new() }
  }

  /// 获取触发器状态
  pub fn get_state(&self, trigger_id: &str) -> Option<&TriggerState> {
    self.states.get(trigger_id)
  }

  /// 更新触发器状态
  pub fn update_state(&mut self, trigger_id: String, state: TriggerState) {
    self.states.insert(trigger_id, state);
  }

  /// 删除触发器状态
  pub fn remove_state(&mut self, trigger_id: &str) {
    self.states.remove(trigger_id);
  }

  /// 清理过期状态
  pub fn cleanup_expired_states(&mut self, max_age_seconds: u64) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);

    self.states.retain(|_, state| current_time.saturating_sub(state.last_updated) < max_age_seconds);
  }
}

/// 触发器状态
#[derive(Debug, Clone)]
pub struct TriggerState {
  /// 最后处理时间戳
  pub last_updated: u64,
  /// 已处理的邮件 UID 集合
  pub processed_uids: std::collections::HashSet<u32>,
  /// 错误计数
  pub error_count: u32,
  /// 最后错误信息
  pub last_error: Option<String>,
}

impl TriggerState {
  /// 创建新的触发器状态
  pub fn new() -> Self {
    Self {
      last_updated: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
      processed_uids: std::collections::HashSet::new(),
      error_count: 0,
      last_error: None,
    }
  }

  /// 添加已处理的邮件 UID
  pub fn add_processed_uid(&mut self, uid: u32) {
    self.processed_uids.insert(uid);
    self.update_timestamp();
  }

  /// 更新时间戳
  pub fn update_timestamp(&mut self) {
    self.last_updated = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
  }

  /// 记录错误
  pub fn record_error(&mut self, error: String) {
    self.error_count += 1;
    self.last_error = Some(error);
    self.update_timestamp();
  }

  /// 重置错误计数
  pub fn reset_errors(&mut self) {
    self.error_count = 0;
    self.last_error = None;
    self.update_timestamp();
  }
}

#[cfg(test)]
mod tests {
  use crate::trigger::email_trigger::AttachmentOptions;

  use super::*;

  #[test]
  fn test_pattern_matching() {
    assert!(matches_pattern("test@example.com", "test@example.com"));
    assert!(matches_pattern("test@example.com", "*@example.com"));
    assert!(matches_pattern("test@example.com", "test@*.com"));
    assert!(matches_pattern("test@example.com", "*@*.com"));
    assert!(matches_pattern("test@example.com", "test@?xample.com"));
    assert!(!matches_pattern("test@example.com", "different@example.com"));
    assert!(!matches_pattern("test@example.com", "test@different.com"));
  }

  #[test]
  fn test_email_filtering() {
    let emails = vec![
      EmailData {
        uid: 1,
        from: Some("sender1@example.com".to_string()),
        to: vec!["recipient@example.com".to_string()],
        subject: Some("Test Subject".to_string()),
        text_body: Some("This is a test email".to_string()),
        html_body: None,
        is_read: false,
        has_attachments: true,
        attachments: vec![],
        message_id: None,
        cc: vec![],
        bcc: vec![],
        sent_date: None,
        received_date: None,
        size: None,
        custom_properties: None,
      },
      EmailData {
        uid: 2,
        from: Some("sender2@example.com".to_string()),
        to: vec!["recipient@example.com".to_string()],
        subject: Some("Different Subject".to_string()),
        text_body: Some("This contains urgent content".to_string()),
        html_body: None,
        is_read: true,
        has_attachments: false,
        attachments: vec![],
        message_id: None,
        cc: vec![],
        bcc: vec![],
        sent_date: None,
        received_date: None,
        size: None,
        custom_properties: None,
      },
    ];

    // 测试发件人过滤
    let filter = EmailFilter {
      from_filter: Some("sender1@example.com".to_string()),
      to_filter: None,
      subject_filter: None,
      content_filter: None,
      read_only_unread: None,
      has_attachments: None,
    };
    let filtered = filter_emails(emails.clone(), &filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].uid, 1);

    // 测试内容过滤
    let filter = EmailFilter {
      from_filter: None,
      to_filter: None,
      subject_filter: None,
      content_filter: Some("urgent".to_string()),
      read_only_unread: None,
      has_attachments: None,
    };
    let filtered = filter_emails(emails.clone(), &filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].uid, 2);

    // 测试只读未读邮件过滤
    let filter = EmailFilter {
      from_filter: None,
      to_filter: None,
      subject_filter: None,
      content_filter: None,
      read_only_unread: Some(true),
      has_attachments: None,
    };
    let filtered = filter_emails(emails.clone(), &filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].uid, 1);

    // 测试附件过滤
    let filter = EmailFilter {
      from_filter: None,
      to_filter: None,
      subject_filter: None,
      content_filter: None,
      read_only_unread: None,
      has_attachments: Some(true),
    };
    let filtered = filter_emails(emails.clone(), &filter);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].uid, 1);
  }

  #[test]
  fn test_email_formatting() {
    let email = EmailData {
      uid: 1,
      message_id: Some("<test@example.com>".to_string()),
      from: Some("sender@example.com".to_string()),
      to: vec!["recipient@example.com".to_string()],
      cc: vec![],
      bcc: vec![],
      subject: Some("Test Subject".to_string()),
      text_body: Some("Test content".to_string()),
      html_body: Some("<p>Test content</p>".to_string()),
      sent_date: Some("2024-01-15T10:30:00Z".to_string()),
      received_date: Some("2024-01-15T10:31:00Z".to_string()),
      attachments: vec![],
      size: Some(1024),
      is_read: false,
      has_attachments: false,
      custom_properties: None,
    };

    // 测试简化格式
    let simple = EmailFormatter::format_email(email.clone(), &EmailReadFormat::Simple).unwrap();
    assert_eq!(simple["uid"], 1);
    assert_eq!(simple["from"], "sender@example.com");
    assert_eq!(simple["subject"], "Test Subject");
    assert_eq!(simple["body"], "Test content");

    // 测试解析格式
    let resolved = EmailFormatter::format_email(email.clone(), &EmailReadFormat::Resolved).unwrap();
    assert_eq!(resolved["uid"], 1);
    assert_eq!(resolved["from"], "sender@example.com");
    assert_eq!(resolved["subject"], "Test Subject");
    assert_eq!(resolved["text_body"], "Test content");
    assert_eq!(resolved["html_body"], "<p>Test content</p>");

    // 测试原始格式
    let raw = EmailFormatter::format_email(email, &EmailReadFormat::Raw).unwrap();
    assert_eq!(raw["uid"], 1);
    assert!(raw["raw_data"].is_string());
  }

  #[test]
  fn test_attachment_processing() {
    let attachments = vec![
      EmailAttachment {
        filename: "document.pdf".to_string(),
        content_type: "application/pdf".to_string(),
        size: 1024,
        data: None,
        content_id: None,
        part_number: None,
      },
      EmailAttachment {
        filename: "image.jpg".to_string(),
        content_type: "image/jpeg".to_string(),
        size: 2048,
        data: None,
        content_id: None,
        part_number: None,
      },
      EmailAttachment {
        filename: "script.exe".to_string(),
        content_type: "application/octet-stream".to_string(),
        size: 512,
        data: None,
        content_id: None,
        part_number: None,
      },
    ];

    // 测试允许的文件类型过滤
    let options = AttachmentOptions {
      download_attachments: Some(true),
      max_attachment_size: None,
      allowed_file_types: Some(vec!["pdf".to_string(), "jpg".to_string()]),
      forbidden_file_types: None,
    };

    let processed = AttachmentProcessor::process_attachments(attachments.clone(), &options).unwrap();
    assert_eq!(processed.len(), 2);
    assert!(
      processed
        .iter()
        .all(|att| ["pdf", "jpg"].contains(&AttachmentProcessor::get_file_extension(&att.filename).as_str()))
    );

    // 测试禁止的文件类型过滤
    let options = AttachmentOptions {
      download_attachments: Some(true),
      max_attachment_size: None,
      allowed_file_types: None,
      forbidden_file_types: Some(vec!["exe".to_string()]),
    };

    let processed = AttachmentProcessor::process_attachments(attachments.clone(), &options).unwrap();
    assert_eq!(processed.len(), 2);
    assert!(!processed.iter().any(|att| att.filename.ends_with(".exe")));

    // 测试大小限制
    let options = AttachmentOptions {
      download_attachments: Some(true),
      max_attachment_size: Some(1500),
      allowed_file_types: None,
      forbidden_file_types: None,
    };

    let processed = AttachmentProcessor::process_attachments(attachments, &options).unwrap();
    assert_eq!(processed.len(), 2); // 只有 PDF (1024) 通过，JPG (2048) 和 EXE (512) 中只有 PDF 通过
  }

  #[test]
  fn test_trigger_state_manager() {
    let mut manager = TriggerStateManager::new();

    // 测试添加状态
    let trigger_id = "test_trigger".to_string();
    let state = TriggerState::new();
    manager.update_state(trigger_id.clone(), state);

    // 测试获取状态
    let retrieved_state = manager.get_state(&trigger_id);
    assert!(retrieved_state.is_some());

    // 测试更新状态
    let mut state = retrieved_state.unwrap().clone();
    state.add_processed_uid(1001);
    manager.update_state(trigger_id.clone(), state);

    let updated_state = manager.get_state(&trigger_id).unwrap();
    assert!(updated_state.processed_uids.contains(&1001));

    // 测试删除状态
    manager.remove_state(&trigger_id);
    assert!(manager.get_state(&trigger_id).is_none());
  }
}
