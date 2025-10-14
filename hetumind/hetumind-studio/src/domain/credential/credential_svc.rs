use axum::extract::FromRequestParts;
use fusion_core::application::Application;
use fusion_core::security::jose::{decrypt_jwe_ecdh_es, encrypt_jwe_ecdh_es};
use fusion_core::{DataError, Result};
use fusion_db::ModelManager;
use fusion_web::{WebError, extract_ctx};
use fusionsql::common::now_offset;
use fusionsql::page::PageResult;
use hetumind_core::workflow::CredentialKind;
use http::request::Parts;
use josekit::jwt::JwtPayload;
use serde_json;
use uuid::Uuid;

use crate::domain::credential::{
  CredentialData, CredentialEntity, CredentialForInsert, CredentialForQuery, CredentialForUpdate,
  CredentialVerifyResult, CredentialWithDecryptedData,
};
use crate::infra::bmc::CredentialBmc;
use crate::infra::security::encryption::EncryptionKeyManager;

/// 凭证服务
pub struct CredentialSvc {
  mm: ModelManager,
  key_manager: EncryptionKeyManager,
}

impl CredentialSvc {
  /// 创建凭证（自动加密敏感数据）
  pub async fn create_credential(&self, mut input: CredentialForInsert) -> Result<Uuid> {
    // 加密凭证数据
    let data = CredentialData { data: input.data, test_connection: true };
    let encrypted_data = self.encrypt_credential_data(&data)?;
    input.data = encrypted_data;

    let id = if input.id.is_none() {
      let id = Uuid::now_v7();
      input.id = Some(id);
      id
    } else {
      input.id.unwrap()
    };

    CredentialBmc::insert(&self.mm, input).await?;
    Ok(id)
  }

  /// 获取凭证（自动解密）
  pub async fn get_credential(&self, id: &Uuid) -> Result<CredentialWithDecryptedData> {
    // TODO 需要检验当前用户是否有相应凭证的访问权限
    let credential = CredentialBmc::find_by_id(&self.mm, id).await?;

    // 解密凭证数据
    let decrypted_data = self.decrypt_credential_data(&credential.data)?;

    Ok(CredentialWithDecryptedData { credential, decrypted_data })
  }

  /// 更新凭证
  pub async fn update_credential(&self, id: &Uuid, mut input: CredentialForUpdate) -> Result<()> {
    // TODO 需要检验当前用户是否有相应凭证的访问权限

    if let Some(data) = input.data.take() {
      let credential_data = CredentialData { data, test_connection: true };
      let encrypted_data = self.encrypt_credential_data(&credential_data)?;
      input.data = Some(encrypted_data);
    }

    CredentialBmc::update_by_id(&self.mm, id, input).await?;
    Ok(())
  }

  /// 删除凭证
  pub async fn delete_credential(&self, id: &Uuid) -> Result<()> {
    CredentialBmc::delete_by_id(&self.mm, id).await?;
    Ok(())
  }

  /// 列出凭证（不包含敏感数据）
  pub async fn query_credentials(&self, query: CredentialForQuery) -> Result<PageResult<CredentialEntity>> {
    let result = CredentialBmc::page(&self.mm, query.filters, query.page).await?;
    Ok(result)
  }

  /// 验证凭证
  pub async fn test_credential(&self, data: &CredentialData, kind: &CredentialKind) -> Result<CredentialVerifyResult> {
    let verify_time = now_offset();

    // 根据凭证类型进行验证
    let (success, message) = match kind {
      CredentialKind::Oauth2 => self.verify_oauth2(data).await?,
      CredentialKind::Authenticate => self.verify_authenticate(data).await?,
      CredentialKind::GenericAuth => self.verify_generic_auth(data).await?,
    };

    Ok(CredentialVerifyResult { success, message, verify_time })
  }

  // --- 私有方法 ---

  /// 加密凭证数据
  fn encrypt_credential_data(&self, data: &CredentialData) -> Result<String> {
    let keys = self.key_manager.get_or_create_encryption_keys()?;

    let json_data = serde_json::to_string(data)
      .map_err(|e| DataError::server_error(format!("Failed to serialize credential data: {}", e)))?;

    // 创建 JWT payload
    let mut payload = JwtPayload::new();
    payload
      .set_claim("data", Some(serde_json::Value::String(json_data)))
      .map_err(|e| DataError::server_error(format!("Failed to set JWT claim: {}", e)))?;

    // 使用 ECDH-ES 加密
    let encrypted = encrypt_jwe_ecdh_es(keys.public_key.as_bytes(), &payload)
      .map_err(|e| DataError::server_error(format!("Failed to encrypt credential data: {}", e)))?;

    Ok(encrypted)
  }

  /// 解密凭证数据
  fn decrypt_credential_data(&self, encrypted_data: &str) -> Result<CredentialData> {
    let keys = self.key_manager.get_or_create_encryption_keys()?;

    // 使用 ECDH-ES 解密
    let (payload, _header) = decrypt_jwe_ecdh_es(keys.private_key.as_bytes(), encrypted_data)
      .map_err(|e| DataError::server_error(format!("Failed to decrypt credential data: {}", e)))?;

    // 提取数据
    let data_value = payload
      .claim("data")
      .and_then(|v| v.as_str())
      .ok_or_else(|| DataError::server_error("Invalid credential data format"))?;

    let credential_data: CredentialData = serde_json::from_str(data_value)
      .map_err(|e| DataError::server_error(format!("Failed to deserialize credential data: {}", e)))?;

    Ok(credential_data)
  }

  // --- 凭证验证方法 ---

  async fn verify_oauth2(&self, _data: &CredentialData) -> Result<(bool, String)> {
    // TODO: 实现 OAuth2 验证
    // 1. 从 data 中提取 OAuth2 配置
    // 2. 进行 OAuth2 流程测试
    // 3. 返回验证结果
    Ok((true, "OAuth2 verification placeholder".to_string()))
  }

  async fn verify_authenticate(&self, _data: &CredentialData) -> Result<(bool, String)> {
    // TODO: 实现通用身份认证验证
    // 1. 从 data 中提取认证信息
    // 2. 进行身份认证测试
    // 3. 返回验证结果
    Ok((true, "Authenticate verification placeholder".to_string()))
  }

  async fn verify_generic_auth(&self, _data: &CredentialData) -> Result<(bool, String)> {
    // TODO: 实现通用认证验证
    // 1. 从 data 中提取通用认证配置
    // 2. 进行认证测试
    // 3. 返回验证结果
    Ok((true, "GenericAuth verification placeholder".to_string()))
  }
}

impl FromRequestParts<Application> for CredentialSvc {
  type Rejection = WebError;

  async fn from_request_parts(parts: &mut Parts, state: &Application) -> core::result::Result<Self, Self::Rejection> {
    let ctx = extract_ctx(parts, state.fusion_config().security())?;
    let mm = state.component::<ModelManager>().with_ctx(ctx);
    let key_manager = state.component();
    Ok(CredentialSvc { mm, key_manager })
  }
}
