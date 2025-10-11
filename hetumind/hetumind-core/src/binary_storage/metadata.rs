//! 二进制数据元数据

use serde::{Deserialize, Serialize};

use crate::types::BinaryFileKind;

/// 二进制数据元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryDataMetadata {
  /// 文件名
  pub file_name: Option<String>,
  /// MIME类型
  pub mime_type: String,
  /// 文件大小
  pub file_size: u64,
  /// 最后修改时间
  pub last_modified: Option<i64>,
  /// 文件类型
  pub file_kind: Option<BinaryFileKind>,
  /// 文件扩展名
  pub file_extension: Option<String>,
  /// 文件目录
  pub directory: Option<String>,
}

impl BinaryDataMetadata {
  /// 创建新的元数据
  pub fn new(file_name: Option<String>, mime_type: String, file_size: u64) -> Self {
    Self {
      file_name,
      mime_type,
      file_size,
      last_modified: None,
      file_kind: None,
      file_extension: None,
      directory: None,
    }
  }

  /// 设置最后修改时间
  pub fn with_last_modified(mut self, last_modified: i64) -> Self {
    self.last_modified = Some(last_modified);
    self
  }

  /// 设置文件类型
  pub fn with_file_kind(mut self, file_kind: BinaryFileKind) -> Self {
    self.file_kind = Some(file_kind);
    self
  }

  /// 设置文件扩展名
  pub fn with_file_extension(mut self, file_extension: String) -> Self {
    self.file_extension = Some(file_extension);
    self
  }

  /// 设置文件目录
  pub fn with_directory(mut self, directory: String) -> Self {
    self.directory = Some(directory);
    self
  }

  /// 根据MIME类型和文件名推断文件类型和扩展名
  pub fn infer_type_and_extension(&mut self) {
    if self.file_kind.is_none() {
      self.file_kind = Some(self.determine_file_kind(&self.mime_type));
    }

    if self.file_extension.is_none()
      && let Some(ref file_name) = self.file_name
    {
      self.file_extension =
        std::path::Path::new(file_name).extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_string());
    }

    if self.directory.is_none()
      && let Some(ref file_name) = self.file_name
    {
      self.directory = std::path::Path::new(file_name)
        .parent()
        .and_then(|parent| parent.to_str())
        .map(|path| path.to_string());
    }
  }

  /// 根据MIME类型确定文件类型
  fn determine_file_kind(&self, mime_type: &str) -> BinaryFileKind {
    match mime_type {
      t if t.starts_with("text/") => BinaryFileKind::Text,
      "application/json" => BinaryFileKind::Json,
      t if t.starts_with("image/") => BinaryFileKind::Image,
      t if t.starts_with("video/") => BinaryFileKind::Video,
      t if t.starts_with("audio/") => BinaryFileKind::Audio,
      "application/pdf" => BinaryFileKind::Pdf,
      "text/html" => BinaryFileKind::Html,
      t if t.contains("sheet") || t.contains("excel") => BinaryFileKind::Excel,
      t if t.contains("word") || t.contains("document") => BinaryFileKind::Word,
      t if t.contains("presentation") || t.contains("powerpoint") => BinaryFileKind::Ppt,
      _ => BinaryFileKind::Text,
    }
  }
}

impl Default for BinaryDataMetadata {
  fn default() -> Self {
    Self::new(None, "application/octet-stream".to_string(), 0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_metadata_creation() {
    let metadata = BinaryDataMetadata::new(Some("test.txt".to_string()), "text/plain".to_string(), 1024);

    assert_eq!(metadata.file_name, Some("test.txt".to_string()));
    assert_eq!(metadata.mime_type, "text/plain");
    assert_eq!(metadata.file_size, 1024);
  }

  #[test]
  fn test_metadata_with_methods() {
    let metadata = BinaryDataMetadata::new(Some("test.txt".to_string()), "text/plain".to_string(), 1024)
      .with_last_modified(1234567890)
      .with_file_kind(BinaryFileKind::Text)
      .with_file_extension("txt".to_string())
      .with_directory("/path/to".to_string());

    assert_eq!(metadata.last_modified, Some(1234567890));
    assert_eq!(metadata.file_kind, Some(BinaryFileKind::Text));
    assert_eq!(metadata.file_extension, Some("txt".to_string()));
    assert_eq!(metadata.directory, Some("/path/to".to_string()));
  }

  #[test]
  fn test_infer_type_and_extension() {
    let mut metadata = BinaryDataMetadata::new(Some("document.pdf".to_string()), "application/pdf".to_string(), 2048);

    metadata.infer_type_and_extension();

    assert_eq!(metadata.file_kind, Some(BinaryFileKind::Pdf));
    assert_eq!(metadata.file_extension, Some("pdf".to_string()));
    assert_eq!(metadata.directory, Some("document".to_string()));
  }

  #[test]
  fn test_determine_file_kind() {
    let metadata = BinaryDataMetadata::default();

    assert_eq!(metadata.determine_file_kind("text/plain"), BinaryFileKind::Text);
    assert_eq!(metadata.determine_file_kind("application/json"), BinaryFileKind::Json);
    assert_eq!(metadata.determine_file_kind("image/jpeg"), BinaryFileKind::Image);
    assert_eq!(metadata.determine_file_kind("video/mp4"), BinaryFileKind::Video);
    assert_eq!(metadata.determine_file_kind("audio/mp3"), BinaryFileKind::Audio);
    assert_eq!(metadata.determine_file_kind("application/pdf"), BinaryFileKind::Pdf);
    assert_eq!(metadata.determine_file_kind("text/html"), BinaryFileKind::Html);
    assert_eq!(metadata.determine_file_kind("application/vnd.ms-excel"), BinaryFileKind::Excel);
    assert_eq!(metadata.determine_file_kind("application/msword"), BinaryFileKind::Word);
    assert_eq!(metadata.determine_file_kind("application/vnd.ms-powerpoint"), BinaryFileKind::Ppt);
  }
}
