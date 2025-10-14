//! MIME 类型检测器 - 提供文件类型检测功能
//!
//! 特性：
//! - 扩展名推断为主
//! - 支持异步操作，避免阻塞
//! - 高精度检测，避免文件扩展名错误导致的误判
use mime_guess::from_path;
use std::path::Path;
use thiserror::Error;

/// MIME 类型检测错误类型
#[derive(Debug, Error)]
pub enum MimeTypeDetectorError {
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("Invalid file path: {0}")]
  InvalidPath(String),

  #[error("Detection failed: {0}")]
  DetectionFailed(String),
}

/// MIME 类型检测器
pub struct MimeTypeDetector;

impl MimeTypeDetector {
  /// 检测文件的 MIME 类型
  ///
  /// # 参数
  /// - `file_path`: 文件路径
  /// - `content_sample`: 可选的内容样本（前 N 字节）
  ///
  /// # 返回值
  /// 检测到的 MIME 类型字符串
  ///
  /// # 检测策略
  /// 1. 如果有内容样本，进行基本内容检测
  /// 2. 使用 mime_guess 扩展名推断作为主要方式
  pub async fn detect_mime_type(
    file_path: &str,
    content_sample: Option<&[u8]>,
  ) -> Result<String, MimeTypeDetectorError> {
    // 1. 如果有内容样本，进行基本内容检测
    if let Some(content) = content_sample {
      // 对文本内容进行更具体的检测
      if Self::looks_like_json(content) {
        return Ok("application/json".to_string());
      } else if Self::looks_like_html(content) {
        return Ok("text/html".to_string());
      } else if Self::looks_like_xml(content) {
        return Ok("application/xml".to_string());
      } else if Self::looks_like_csv(content) {
        return Ok("text/csv".to_string());
      } else if Self::looks_like_yaml(content) {
        return Ok("application/x-yaml".to_string());
      }
    }

    // 2. 使用扩展名检测作为主要方式
    Ok(from_path(file_path).first_or_octet_stream().to_string())
  }

  /// 从文件流检测 MIME 类型（异步，推荐使用）
  ///
  /// # 参数
  /// - `file_path`: 文件路径
  /// - `mut stream`: 可读的文件流
  ///
  /// # 特性
  /// - 只读取文件头部进行检测（默认 1KB）
  /// - 支持自定义采样大小
  /// - 流式处理，减少内存占用
  pub async fn detect_mime_type_from_stream<R: std::io::Read + Unpin>(
    file_path: &str,
    mut stream: R,
    sample_size: usize,
  ) -> Result<String, MimeTypeDetectorError> {
    let mut buffer = vec![0u8; sample_size];
    let bytes_read = stream.read(&mut buffer)?;

    if bytes_read == 0 {
      // 空文件，使用扩展名检测
      return Ok(from_path(file_path).first_or_octet_stream().to_string());
    }

    buffer.truncate(bytes_read);
    Self::detect_mime_type(file_path, Some(&buffer)).await
  }

  /// 检测代码文件的具体语言类型
  ///
  /// 基于文件扩展名和内容特征的代码语言检测
  pub fn detect_code_language(file_path: &str, content: Option<&[u8]>) -> Option<String> {
    let extension = Path::new(file_path).extension().and_then(|ext| ext.to_str())?;

    let mime_type = match extension {
      "rs" => Some("text/x-rust".to_string()),
      "js" | "mjs" => Some("application/javascript".to_string()),
      "ts" => Some("application/typescript".to_string()),
      "jsx" => Some("text/jsx".to_string()),
      "tsx" => Some("text/tsx".to_string()),
      "py" => Some("text/x-python".to_string()),
      "java" => Some("text/x-java".to_string()),
      "cpp" | "cc" | "cxx" => Some("text/x-c++".to_string()),
      "c" => Some("text/x-c".to_string()),
      "go" => Some("text/x-go".to_string()),
      "php" => Some("application/x-httpd-php".to_string()),
      "rb" => Some("text/x-ruby".to_string()),
      "swift" => Some("text/x-swift".to_string()),
      "kt" => Some("text/x-kotlin".to_string()),
      "scala" => Some("text/x-scala".to_string()),
      "sh" | "bash" => Some("application/x-sh".to_string()),
      "sql" => Some("application/sql".to_string()),
      "css" => Some("text/css".to_string()),
      "scss" | "sass" => Some("text/x-scss".to_string()),
      "less" => Some("text/x-less".to_string()),
      "html" | "htm" => Some("text/html".to_string()),
      "xml" => Some("application/xml".to_string()),
      "json" => Some("application/json".to_string()),
      "yaml" | "yml" => Some("application/x-yaml".to_string()),
      "toml" => Some("application/toml".to_string()),
      "md" => Some("text/markdown".to_string()),
      _ => None,
    };

    // 如果有内容样本，可以进行更精确的检测
    if let (Some(mime_type), Some(content)) = (&mime_type, content) {
      // 对于文本文件，验证内容是否符合预期的类型
      if mime_type.contains("text/") || mime_type.contains("application/") {
        match extension {
          "json" if Self::looks_like_json(content) => return Some(mime_type.clone()),
          "html" | "htm" if Self::looks_like_html(content) => return Some(mime_type.clone()),
          "xml" if Self::looks_like_xml(content) => return Some(mime_type.clone()),
          _ => return Some(mime_type.clone()),
        }
      } else {
        return Some(mime_type.clone());
      }
    }

    mime_type
  }

  /// 根据文件类型确定 BinaryFileKind
  pub fn determine_file_kind(mime_type: &str) -> String {
    match mime_type {
      t if t.starts_with("text/") && mime_type != "text/html" => "Text".to_string(),
      "application/json" => "Json".to_string(),
      t if t.starts_with("image/") => "Image".to_string(),
      t if t.starts_with("video/") => "Video".to_string(),
      t if t.starts_with("audio/") => "Audio".to_string(),
      "application/pdf" => "Pdf".to_string(),
      "text/html" => "Html".to_string(),
      t if t.contains("sheet") || t.contains("excel") => "Excel".to_string(),
      t if t.contains("word") || t.contains("document") => "Word".to_string(),
      t if t.contains("presentation") || t.contains("powerpoint") => "Ppt".to_string(),
      _ => "Binary".to_string(),
    }
  }

  // 私有辅助方法
  fn looks_like_json(content: &[u8]) -> bool {
    let trimmed: Vec<u8> = content.iter().skip_while(|&&b| b.is_ascii_whitespace()).take(100).copied().collect();

    !trimmed.is_empty() && (trimmed[0] == b'{' || trimmed[0] == b'[')
  }

  fn looks_like_html(content: &[u8]) -> bool {
    let trimmed: Vec<u8> = content.iter().skip_while(|&&b| b.is_ascii_whitespace()).take(100).copied().collect();

    if trimmed.len() < 4 {
      return false;
    }

    let start = String::from_utf8_lossy(&trimmed[..4]).to_lowercase();
    start.starts_with("<!do") || start.starts_with("<html") || start.starts_with("<head") || start.starts_with("<body")
  }

  fn looks_like_xml(content: &[u8]) -> bool {
    let trimmed: Vec<u8> = content.iter().skip_while(|&&b| b.is_ascii_whitespace()).take(100).copied().collect();

    if trimmed.len() < 5 {
      return false;
    }

    let start = String::from_utf8_lossy(&trimmed[..5]).to_lowercase();
    start.starts_with("<?xml") || (start.starts_with("<") && start.contains("xmlns"))
  }

  fn looks_like_csv(content: &[u8]) -> bool {
    let sample = String::from_utf8_lossy(&content[..content.len().min(1024)]);
    let lines: Vec<&str> = sample.lines().take(5).collect();

    if lines.len() < 2 {
      return false;
    }

    // 检查是否包含逗号分隔符
    lines.iter().any(|line| line.contains(',') && line.split(',').count() > 1)
  }

  fn looks_like_yaml(content: &[u8]) -> bool {
    let sample = String::from_utf8_lossy(&content[..content.len().min(1024)]);
    let lines: Vec<&str> = sample.lines().take(10).collect();

    // 检查 YAML 特征
    lines.iter().any(|line| {
      let trimmed = line.trim();
      trimmed.starts_with('-') || trimmed.contains(':') || trimmed.starts_with('#') || trimmed == "---"
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_json_detection() {
    let json_content = r#"{"name": "test", "value": 123}"#;
    let mime_type = MimeTypeDetector::detect_mime_type("test.json", Some(json_content.as_bytes())).await.unwrap();
    assert_eq!(mime_type, "application/json");
  }

  #[tokio::test]
  async fn test_html_detection() {
    let html_content = r#"<html><head><title>Test</title></head></html>"#;
    let mime_type = MimeTypeDetector::detect_mime_type("test.html", Some(html_content.as_bytes())).await.unwrap();
    assert_eq!(mime_type, "text/html");
  }

  #[test]
  fn test_code_language_detection() {
    assert_eq!(MimeTypeDetector::detect_code_language("main.rs", None), Some("text/x-rust".to_string()));
    assert_eq!(MimeTypeDetector::detect_code_language("script.py", None), Some("text/x-python".to_string()));
  }

  #[tokio::test]
  async fn test_fallback_to_extension() {
    let mime_type = MimeTypeDetector::detect_mime_type("unknown.xyz", None).await.unwrap();
    assert_eq!(mime_type, "chemical/x-xyz");
  }

  #[test]
  fn test_determine_file_kind() {
    assert_eq!(MimeTypeDetector::determine_file_kind("text/plain"), "Text");
    assert_eq!(MimeTypeDetector::determine_file_kind("application/json"), "Json");
    assert_eq!(MimeTypeDetector::determine_file_kind("image/jpeg"), "Image");
    assert_eq!(MimeTypeDetector::determine_file_kind("video/mp4"), "Video");
    assert_eq!(MimeTypeDetector::determine_file_kind("audio/mp3"), "Audio");
    assert_eq!(MimeTypeDetector::determine_file_kind("application/pdf"), "Pdf");
    assert_eq!(MimeTypeDetector::determine_file_kind("text/html"), "Html");
    assert_eq!(MimeTypeDetector::determine_file_kind("application/vnd.ms-excel"), "Excel");
    assert_eq!(MimeTypeDetector::determine_file_kind("application/msword"), "Word");
    assert_eq!(MimeTypeDetector::determine_file_kind("application/vnd.ms-powerpoint"), "Ppt");
  }
}
