//! Read/Write Files Node 工具函数
//!
//! 提供文件读取、写入和错误处理的实用工具函数。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use glob::glob;
use hetumind_core::{
  types::{BinaryFileKind, JsonValue},
  utils::mime_detector::MimeTypeDetector,
  workflow::{BinaryDataReference, NodeExecutionContext, NodeExecutionError},
};
use serde_json::json;
use tokio::fs;

use super::FileErrorContext;

/// 文件读取器
pub struct FileReader;

impl FileReader {
  /// 读取单个文件并创建二进制数据引用
  pub async fn read_file_to_binary_reference(
    file_path: &str,
    _context: &NodeExecutionContext,
  ) -> Result<BinaryDataReference, NodeExecutionError> {
    // 读取文件内容
    let file_content = fs::read(file_path)
      .await
      .map_err(|e| NodeExecutionError::DataProcessingError { message: format!("Failed to read file: {}", e) })?;

    // 获取文件元数据
    let metadata = fs::metadata(file_path).await.map_err(|e| NodeExecutionError::DataProcessingError {
      message: format!("Failed to get file metadata: {}", e),
    })?;

    // 使用核心 MIME 类型检测工具函数
    let mime_type = Self::detect_mime_type(file_path, &file_content).await?;

    // 确定文件类型
    let file_kind = Self::determine_file_kind(&mime_type);

    // 创建文件路径对象以提取文件信息
    let path_obj = Path::new(file_path);
    let file_name = path_obj.file_name().and_then(|name| name.to_str()).map(|name| name.to_string());
    let file_extension = path_obj.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_string());
    let directory = path_obj.parent().and_then(|parent| parent.to_str()).map(|path| path.to_string());

    // 创建二进制数据引用
    let binary_ref = BinaryDataReference {
      file_key: Self::generate_file_key(file_path).await?,
      mime_kind: mime_type.clone(),
      file_size: metadata.len(),
      file_name,
      file_kind: Some(file_kind),
      file_extension,
      directory,
    };

    Ok(binary_ref)
  }

  /// 使用核心 MIME 类型检测工具函数
  async fn detect_mime_type(file_path: &str, file_content: &[u8]) -> Result<String, NodeExecutionError> {
    // 调用核心工具函数进行 MIME 类型检测
    MimeTypeDetector::detect_mime_type(file_path, Some(file_content))
      .await
      .map_err(|e| NodeExecutionError::DataProcessingError { message: format!("Failed to detect MIME type: {}", e) })
  }

  /// 确定文件类型
  fn determine_file_kind(mime_type: &str) -> BinaryFileKind {
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

  /// 生成文件键
  async fn generate_file_key(file_path: &str) -> Result<String, NodeExecutionError> {
    // 使用文件路径和修改时间生成哈希
    let metadata = fs::metadata(file_path).await.map_err(|e| NodeExecutionError::DataProcessingError {
      message: format!("Failed to get file metadata for key generation: {}", e),
    })?;

    let modified = metadata.modified().map_err(|e| NodeExecutionError::DataProcessingError {
      message: format!("Failed to get file modification time: {}", e),
    })?;

    let mut hasher = DefaultHasher::new();
    file_path.hash(&mut hasher);
    modified.hash(&mut hasher);

    Ok(format!("file_{}", hasher.finish()))
  }

  /// 使用 glob 模式匹配文件
  pub async fn match_files(pattern: &str) -> Result<Vec<String>, NodeExecutionError> {
    let pattern = Self::escape_glob_pattern(pattern);

    let mut matched_files = Vec::new();

    for entry in glob(&pattern)
      .map_err(|e| NodeExecutionError::DataProcessingError { message: format!("Invalid glob pattern: {}", e) })?
    {
      match entry {
        Ok(path) => {
          if let Some(path_str) = path.to_str() {
            matched_files.push(path_str.to_string());
          }
        }
        Err(e) => {
          log::warn!("Error while reading file entry: {}", e);
        }
      }
    }

    Ok(matched_files)
  }

  /// 转义 glob 模式中的特殊字符
  fn escape_glob_pattern(pattern: &str) -> String {
    pattern.replace('(', "\\(").replace(')', "\\)").replace('[', "\\[").replace(']', "\\]")
  }

  /// 创建文件元数据 JSON
  pub fn create_file_metadata(binary_ref: &BinaryDataReference, file_path: &str) -> JsonValue {
    json!({
        "fileName": binary_ref.file_name,
        "filePath": file_path,
        "fileSize": binary_ref.file_size,
        "mimeType": binary_ref.mime_kind,
        "fileExtension": binary_ref.file_extension,
        "fileType": binary_ref.file_kind,
        "directory": binary_ref.directory,
    })
  }
}

/// 文件写入器
pub struct FileWriter;

impl FileWriter {
  /// 从二进制数据引用获取文件内容（模拟实现）
  pub async fn get_file_content_from_binary_ref(
    _binary_ref: &BinaryDataReference,
    _context: &NodeExecutionContext,
  ) -> Result<Vec<u8>, NodeExecutionError> {
    // 在真实实现中，这里应该从二进制数据管理器获取文件内容
    // 由于我们目前没有完整的二进制数据管理器实现，这里返回错误
    Err(NodeExecutionError::DataProcessingError { message: "Binary data manager not fully implemented".to_string() })
  }

  /// 写入文件到磁盘
  pub async fn write_file_to_disk(
    file_path: &str,
    content: Vec<u8>,
    append_mode: bool,
  ) -> Result<(), NodeExecutionError> {
    // 确保父目录存在
    if let Some(parent) = Path::new(file_path).parent() {
      fs::create_dir_all(parent).await.map_err(|e| NodeExecutionError::DataProcessingError {
        message: format!("Failed to create directory: {}", e),
      })?;
    }

    // 根据模式选择写入方式
    if append_mode {
      // 追加模式
      use tokio::io::AsyncWriteExt;
      let mut file = tokio::fs::OpenOptions::new().create(true).append(true).open(file_path).await.map_err(|e| {
        NodeExecutionError::DataProcessingError { message: format!("Failed to open file for writing: {}", e) }
      })?;

      file
        .write_all(&content)
        .await
        .map_err(|e| NodeExecutionError::DataProcessingError { message: format!("Failed to write to file: {}", e) })?;
    } else {
      // 覆盖模式
      fs::write(file_path, content)
        .await
        .map_err(|e| NodeExecutionError::DataProcessingError { message: format!("Failed to write file: {}", e) })?;
    }

    Ok(())
  }

  /// 创建或更新二进制数据引用（简化实现）
  pub async fn create_or_update_binary_ref(file_path: &str) -> Result<BinaryDataReference, NodeExecutionError> {
    // 读取写入后的文件内容
    let file_content = fs::read(file_path).await.map_err(|e| NodeExecutionError::DataProcessingError {
      message: format!("Failed to read written file: {}", e),
    })?;

    // 获取文件元数据
    let metadata = fs::metadata(file_path).await.map_err(|e| NodeExecutionError::DataProcessingError {
      message: format!("Failed to get written file metadata: {}", e),
    })?;

    // 使用 MIME 类型检测
    let mime_type = MimeTypeDetector::detect_mime_type(file_path, Some(&file_content))
      .await
      .map_err(|e| NodeExecutionError::DataProcessingError { message: format!("Failed to detect MIME type: {}", e) })?;

    // 确定文件类型
    let file_kind = Self::determine_file_kind(&mime_type);

    // 创建文件路径对象以提取文件信息
    let path_obj = Path::new(file_path);
    let file_name = path_obj.file_name().and_then(|name| name.to_str()).map(|name| name.to_string());
    let file_extension = path_obj.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_string());
    let directory = path_obj.parent().and_then(|parent| parent.to_str()).map(|path| path.to_string());

    // 创建二进制数据引用
    let binary_ref = BinaryDataReference {
      file_key: Self::generate_file_key(file_path).await?,
      mime_kind: mime_type.clone(),
      file_size: metadata.len(),
      file_name,
      file_kind: Some(file_kind),
      file_extension,
      directory,
    };

    Ok(binary_ref)
  }

  /// 确定文件类型
  fn determine_file_kind(mime_type: &str) -> BinaryFileKind {
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

  /// 生成文件键
  async fn generate_file_key(file_path: &str) -> Result<String, NodeExecutionError> {
    // 使用文件路径和当前时间生成哈希
    let mut hasher = DefaultHasher::new();
    file_path.hash(&mut hasher);
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs()
      .hash(&mut hasher);

    Ok(format!("file_{}", hasher.finish()))
  }

  /// 创建文件元数据 JSON
  pub fn create_file_metadata(binary_ref: &BinaryDataReference, file_path: &str, append_mode: bool) -> JsonValue {
    json!({
        "fileName": binary_ref.file_name,
        "filePath": file_path,
        "fileSize": binary_ref.file_size,
        "mimeType": binary_ref.mime_kind,
        "fileExtension": binary_ref.file_extension,
        "fileType": binary_ref.file_kind,
        "directory": binary_ref.directory,
        "appendMode": append_mode,
    })
  }
}

/// 文件操作错误映射器
pub struct FileErrorMapper;

impl FileErrorMapper {
  /// 映射文件系统错误到用户友好的错误信息
  pub fn map_file_error(error: &std::io::Error, context: &FileErrorContext) -> NodeExecutionError {
    match error.kind() {
      std::io::ErrorKind::PermissionDenied => {
        let message = if context.operation == "read" {
          format!("您没有权限访问文件 {}", context.file_path)
        } else {
          format!("您没有权限写入文件 {}", context.file_path)
        };

        NodeExecutionError::DataProcessingError { message }
      }
      std::io::ErrorKind::NotFound => {
        NodeExecutionError::DataProcessingError { message: format!("文件不存在: {}", context.file_path) }
      }
      std::io::ErrorKind::AlreadyExists => {
        NodeExecutionError::DataProcessingError { message: format!("文件已存在: {}", context.file_path) }
      }
      std::io::ErrorKind::InvalidInput => {
        NodeExecutionError::DataProcessingError { message: format!("无效的文件路径: {}", context.file_path) }
      }
      _ => NodeExecutionError::DataProcessingError { message: format!("文件操作失败: {}", error) },
    }
  }
}
