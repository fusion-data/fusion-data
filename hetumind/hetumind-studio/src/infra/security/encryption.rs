use chrono::{DateTime, FixedOffset, Utc};
use fusion_core::{DataError, Result};
use openssl::{
  ec::{EcGroup, EcKey},
  nid::Nid,
  pkey::PKey,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zeroize::Zeroize;

/// 加密密钥管理器
#[derive(Clone)]
pub struct EncryptionKeyManager {
  key_file_path: PathBuf,
}

/// 加密密钥结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKeys {
  pub current_version: u32,
  pub private_key: String,
  pub public_key: String,
  pub created_at: DateTime<FixedOffset>,
}

impl EncryptionKeyManager {
  /// 创建新的加密密钥管理器
  pub fn new() -> Self {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let key_file_path = home_dir.join(".hetu").join("config").join("encrypt-key.toml");

    Self { key_file_path }
  }

  /// 获取或创建加密密钥
  pub fn get_or_create_encryption_keys(&self) -> Result<EncryptionKeys> {
    if self.key_file_path.exists() { self.load_existing_keys() } else { self.create_new_keys() }
  }

  /// 轮换密钥（为未来扩展预留）
  pub fn rotate_keys(&self) -> Result<()> {
    // TODO: 实现密钥轮换逻辑
    // 1. 生成新密钥对
    // 2. 读取数据库中所有加密数据
    // 3. 使用新密钥重新加密
    // 4. 更新数据库记录
    // 5. 更新密钥文件
    Err(DataError::server_error("Key rotation not implemented yet"))
  }

  // --- 私有方法 ---

  fn load_existing_keys(&self) -> Result<EncryptionKeys> {
    let content = std::fs::read_to_string(&self.key_file_path)
      .map_err(|e| DataError::server_error(format!("Failed to read key file: {}", e)))?;

    let keys: EncryptionKeys =
      toml::from_str(&content).map_err(|e| DataError::server_error(format!("Failed to parse key file: {}", e)))?;

    Ok(keys)
  }

  fn create_new_keys(&self) -> Result<EncryptionKeys> {
    // 创建目录
    if let Some(parent) = self.key_file_path.parent() {
      std::fs::create_dir_all(parent)
        .map_err(|e| DataError::server_error(format!("Failed to create key directory: {}", e)))?;
    }

    // 生成 EC P-256 密钥对
    let private_key = self.generate_ec_private_key()?;
    let public_key = self.derive_public_key(&private_key)?;

    let keys = EncryptionKeys { current_version: 1, private_key, public_key, created_at: Utc::now().fixed_offset() };

    // 保存到文件
    self.save_keys_to_file(&keys)?;
    self.set_file_permissions()?;

    Ok(keys)
  }

  fn generate_ec_private_key(&self) -> Result<String> {
    // Create EC group for P-256 curve
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)
      .map_err(|e| DataError::server_error(format!("Failed to create EC group: {}", e)))?;

    // Generate EC key pair
    let ec_key =
      EcKey::generate(&group).map_err(|e| DataError::server_error(format!("Failed to generate EC key: {}", e)))?;

    // Convert to PKey format
    let pkey = PKey::from_ec_key(ec_key)
      .map_err(|e| DataError::server_error(format!("Failed to convert EC key to PKey: {}", e)))?;

    // Export private key to PEM format
    let private_pem = pkey
      .private_key_to_pem_pkcs8()
      .map_err(|e| DataError::server_error(format!("Failed to export private key to PEM: {}", e)))?;

    String::from_utf8(private_pem)
      .map_err(|e| DataError::server_error(format!("Failed to convert PEM bytes to string: {}", e)))
  }

  fn derive_public_key(&self, private_key: &str) -> Result<String> {
    // Parse private key from PEM format
    let pkey = PKey::private_key_from_pem(private_key.as_bytes())
      .map_err(|e| DataError::server_error(format!("Failed to parse private key PEM: {}", e)))?;

    // Export public key to PEM format
    let public_pem = pkey
      .public_key_to_pem()
      .map_err(|e| DataError::server_error(format!("Failed to export public key to PEM: {}", e)))?;

    String::from_utf8(public_pem)
      .map_err(|e| DataError::server_error(format!("Failed to convert public key PEM bytes to string: {}", e)))
  }

  fn save_keys_to_file(&self, keys: &EncryptionKeys) -> Result<()> {
    let content =
      toml::to_string_pretty(keys).map_err(|e| DataError::server_error(format!("Failed to serialize keys: {}", e)))?;

    std::fs::write(&self.key_file_path, content)
      .map_err(|e| DataError::server_error(format!("Failed to write key file: {}", e)))?;

    Ok(())
  }

  fn set_file_permissions(&self) -> Result<()> {
    // 跨平台文件权限设置
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mut perms = std::fs::metadata(&self.key_file_path)
        .map_err(|e| DataError::server_error(format!("Failed to get file metadata: {}", e)))?
        .permissions();
      perms.set_mode(0o600); // 仅所有者可读写
      std::fs::set_permissions(&self.key_file_path, perms)
        .map_err(|e| DataError::server_error(format!("Failed to set file permissions: {}", e)))?;
    }

    #[cfg(windows)]
    {
      // Windows 下使用默认权限，但确保文件存在
      let _ = std::fs::metadata(&self.key_file_path)
        .map_err(|e| DataError::server_error(format!("Failed to get file metadata: {}", e)))?;
    }

    Ok(())
  }
}

impl Drop for EncryptionKeyManager {
  fn drop(&mut self) {
    // 清理敏感数据
    if let Some(path_str) = self.key_file_path.to_str() {
      let mut path_string = path_str.to_string();
      path_string.zeroize();
    }
  }
}
