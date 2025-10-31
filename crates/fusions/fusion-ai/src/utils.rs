use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use regex::Regex;
use serde_json::Value;
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageError {
  #[error("Invalid JSON: {0}")]
  InvalidJson(#[from] serde_json::Error),
  #[error("Missing key '{0}' in JSON")]
  MissingKey(String),
  #[error("Invalid Base64: {0}")]
  InvalidBase64(#[from] base64::DecodeError),
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Unsupported image format: {0}")]
  UnsupportedFormat(String),
}

/// 将包含 Base64 图片数据的 JSON 字符串转换为图片文件
///
/// # Arguments
/// * `b64_json_string` - 包含 Base64 图片数据的 JSON 字符串
/// * `output_path` - 保存图片的路径 (例如: "output.png")
/// * `data_key` - JSON 中存储 Base64 数据的键名，默认为 "b64_json"
///
/// # Examples
/// ```
/// use fusion_ai::utils::base64_json_to_image;
///
/// let json_str = r#"{"b64_json": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="}"#;
/// base64_json_to_image(json_str, "output.png", "b64_json")?;
/// ```
pub fn base64_json_to_image(b64_json_string: &str, output_path: &str, data_key: &str) -> Result<(), ImageError> {
  // 解析 JSON
  let data: Value = serde_json::from_str(b64_json_string)?;

  // 获取 Base64 字符串
  let base64_string = data
    .get(data_key)
    .ok_or_else(|| ImageError::MissingKey(data_key.to_string()))?
    .as_str()
    .ok_or_else(|| ImageError::MissingKey(format!("{} is not a string", data_key)))?;

  // 清理 Data URI 前缀 (如果存在)
  let base64_string = clean_data_uri_prefix(base64_string);

  // 解码 Base64
  let image_data = BASE64_STANDARD.decode(base64_string)?;

  // 保存为文件
  write_image_data(&image_data, output_path)?;

  println!("图片成功保存到: {}", output_path);
  Ok(())
}

/// 从 Vec<u8> 生成图片文件并保存到指定位置
///
/// # Arguments
/// * `image_data` - 图片二进制数据
/// * `output_path` - 保存图片的路径，例如 "output.png"
///
/// # Examples
/// ```
/// use fusion_ai::utils::vec_to_image_file;
///
/// let image_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
/// vec_to_image_file(&image_data, "output.png")?;
/// ```
pub fn vec_to_image_file(image_data: &[u8], output_path: &str) -> Result<(), ImageError> {
  write_image_data(image_data, output_path)?;
  println!("图片成功保存到: {}", output_path);
  Ok(())
}

/// 清理 Base64 字符串中的 Data URI 前缀
fn clean_data_uri_prefix(base64_string: &str) -> &str {
  // 使用正则表达式移除 "data:image/...;base64," 前缀
  let re = Regex::new(r"^data:image/\w+;base64,").unwrap();
  re.replace(base64_string, "").into_owned().leak()
}

/// 将图片数据写入文件
fn write_image_data(image_data: &[u8], output_path: &str) -> Result<(), ImageError> {
  let mut file = std::fs::File::create(output_path)?;
  file.write_all(image_data)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_clean_data_uri_prefix() {
    let with_prefix = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
    let without_prefix =
      "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

    assert_eq!(clean_data_uri_prefix(with_prefix), without_prefix);
    assert_eq!(clean_data_uri_prefix(without_prefix), without_prefix);
  }
}
