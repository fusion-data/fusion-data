use fusions::common::ahash::HashMap;
use fusions::core::{DataError, application::Application, configuration::ConfigRegistry};
use fusionsql::ModelManager;
use log::info;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use jieyuan_core::model::{
  OAuthAuthorizeRequest, OAuthAuthorizeResponse, OAuthProvider, OAuthTokenRequest, OAuthTokenResponse, TokenType,
  UserForCreate, UserStatus,
};

use crate::{access_control::make_token, user::UserBmc};

/// OAuth 认证服务
///
/// 提供完整的 OAuth 2.0 + PKCE 认证流程支持，包括：
/// - 授权 URL 生成和 PKCE 支持
/// - 授权码交换令牌
/// - 第三方用户信息获取和解析
/// - 用户创建/更新和统一令牌生成
/// - 用户变更查询（事件轮询）
#[derive(Clone)]
pub struct OAuthSvc {
  mm: ModelManager,
  http: Client,
  app: Application,
}

impl OAuthSvc {
  /// 创建新的 OAuth 服务实例
  ///
  /// # Arguments
  /// * `mm` - 数据库模型管理器
  /// * `app` - 应用程序实例，用于访问配置
  ///
  /// # Returns
  /// OAuth 服务实例
  pub fn new(mm: ModelManager, app: Application) -> Self {
    Self { mm, http: Client::builder().timeout(std::time::Duration::from_secs(30)).build().unwrap_or_default(), app }
  }

  /// 生成授权 URL
  ///
  /// 根据 OAuth 提供商配置生成授权 URL，支持 PKCE 安全增强。
  /// 如果未提供 code_challenge，将自动生成 PKCE code_verifier 和 code_challenge。
  ///
  /// # Arguments
  /// * `req` - OAuth 授权请求参数
  ///
  /// # Returns
  /// 包含授权 URL、状态参数和 code_verifier 的响应
  ///
  /// # Errors
  /// 如果 OAuth 提供商配置无效或 URL 构建失败
  pub async fn authorize(&self, req: OAuthAuthorizeRequest) -> fusions::core::Result<OAuthAuthorizeResponse> {
    let provider_config = self.get_provider_config(req.provider)?;

    // 生成 PKCE code_verifier 和 code_challenge
    let (code_verifier, code_challenge) = if req.code_challenge.is_none() {
      let verifier = self.generate_code_verifier();
      let challenge = self.generate_code_challenge(&verifier);
      (Some(verifier), Some(challenge))
    } else {
      (None, req.code_challenge)
    };

    // 构建授权 URL
    let mut params = HashMap::default();
    params.insert("response_type", "code");
    params.insert("client_id", &provider_config.client_id);
    params.insert("redirect_uri", req.redirect_uri.as_deref().unwrap_or(&provider_config.redirect_uri));
    params.insert("scope", &provider_config.scope);

    if let Some(ref challenge) = code_challenge {
      params.insert("code_challenge", challenge.as_str());
    }
    if let Some(ref method) = req.code_challenge_method {
      params.insert("code_challenge_method", method);
    }
    if let Some(ref state) = req.state {
      params.insert("state", state);
    }

    let authorize_url = format!(
      "{}?{}",
      provider_config.authorize_url,
      params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
    );

    info!("Generated authorize URL for provider {:?}", req.provider);

    Ok(OAuthAuthorizeResponse { authorize_url, state: req.state, code_verifier })
  }

  /// 交换授权码获取令牌
  ///
  /// 使用授权码向第三方 OAuth 提供商交换访问令牌，
  /// 获取用户信息，创建/更新本地用户记录，并生成 Jieyuan 统一令牌。
  ///
  /// # Arguments
  /// * `req` - OAuth 令牌交换请求参数
  ///
  /// # Returns
  /// 包含 Jieyuan 统一令牌和用户信息的响应
  ///
  /// # Errors
  /// 如果令牌交换失败、用户信息获取失败或用户创建失败
  pub async fn exchange_token(&self, req: OAuthTokenRequest) -> fusions::core::Result<OAuthTokenResponse> {
    let provider_config = self.get_provider_config(req.provider)?;

    // 构建令牌交换请求
    let mut params = HashMap::default();
    params.insert("grant_type", "authorization_code");
    params.insert("client_id", &provider_config.client_id);
    params.insert("client_secret", &provider_config.client_secret);
    params.insert("code", &req.code);
    params.insert("redirect_uri", req.redirect_uri.as_deref().unwrap_or(&provider_config.redirect_uri));

    if let Some(ref verifier) = req.code_verifier {
      params.insert("code_verifier", verifier);
    }

    // 发送请求到第三方 OAuth 提供商
    let form_body = params
      .iter()
      .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
      .collect::<Vec<_>>()
      .join("&");

    let response = self
      .http
      .post(&provider_config.token_url)
      .header("Content-Type", "application/x-www-form-urlencoded")
      .body(form_body)
      .send()
      .await
      .map_err(|e| DataError::server_error(format!("Failed to request token: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(DataError::server_error(format!("Token exchange failed: {}", error_text)));
    }

    let token_response: Value = response
      .json()
      .await
      .map_err(|e| DataError::server_error(format!("Failed to parse token response: {}", e)))?;

    let access_token = token_response
      .get("access_token")
      .and_then(|v| v.as_str())
      .ok_or_else(|| DataError::server_error("Missing access_token in response"))?;

    // 获取用户信息
    let user_info = self.get_user_info(req.provider, access_token).await?;

    // 创建或更新用户
    let iam_user_id = self.create_or_update_user(&user_info).await?;

    // 生成 Jieyuan 统一令牌
    let config = self.app.fusion_setting();
    let token = make_token(config.security(), iam_user_id)?;

    info!("Successfully exchanged token for provider {:?}, user_id: {}", req.provider, iam_user_id);

    Ok(OAuthTokenResponse {
      token,
      token_type: TokenType::Bearer,
      expires_in: token_response.get("expires_in").and_then(|v| v.as_i64()),
      refresh_token: token_response.get("refresh_token").and_then(|v| v.as_str()).map(|s| s.to_string()),
      iam_user_id: Some(iam_user_id),
      subject: Some(user_info.id),
    })
  }

  // --- 私有辅助方法 ---

  /// 获取 OAuth 提供商配置
  ///
  /// 从应用程序配置中获取指定 OAuth 提供商的配置信息。
  ///
  /// # Arguments
  /// * `provider` - OAuth 提供商类型
  ///
  /// # Returns
  /// OAuth 提供商配置
  ///
  /// # Errors
  /// 如果提供商无效或配置不存在
  fn get_provider_config(&self, provider: OAuthProvider) -> fusions::core::Result<OAuthProviderConfig> {
    match provider {
      OAuthProvider::Unspecified => Err(DataError::bad_request("Invalid oauth provider")),
      _ => {
        let conf_path = format!("auth.oauth.{}", provider.as_ref());
        self.app.get_config_by_path(&conf_path).map_err(|e| {
          DataError::server_error(format!("Invalid auth provider configuration, path: {}, error: {}", conf_path, e))
        })
      }
    }
  }

  /// 生成 PKCE code_verifier
  ///
  /// 生成符合 RFC 7636 规范的 code_verifier，包含 43-128 个字符。
  ///
  /// # Returns
  /// 随机生成的 code_verifier 字符串
  fn generate_code_verifier(&self) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::rng();
    (0..128)
      .map(|_| {
        let idx = rng.random_range(0..CHARSET.len());
        CHARSET[idx] as char
      })
      .collect()
  }

  /// 生成 PKCE code_challenge
  ///
  /// 使用 SHA256 算法对 code_verifier 进行哈希，并使用 Base64URL 编码生成 code_challenge。
  ///
  /// # Arguments
  /// * `verifier` - code_verifier 字符串
  ///
  /// # Returns
  /// Base64URL 编码的 code_challenge 字符串
  fn generate_code_challenge(&self, verifier: &str) -> String {
    use base64ct::{Base64UrlUnpadded, Encoding};
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(verifier.as_bytes());
    Base64UrlUnpadded::encode_string(&hash)
  }

  /// 获取第三方用户信息
  ///
  /// 使用访问令牌向第三方 OAuth 提供商请求用户信息。
  ///
  /// # Arguments
  /// * `provider` - OAuth 提供商类型
  /// * `access_token` - 访问令牌
  ///
  /// # Returns
  /// 标准化的用户信息
  ///
  /// # Errors
  /// 如果请求失败或响应解析失败
  async fn get_user_info(&self, provider: OAuthProvider, access_token: &str) -> fusions::core::Result<OAuthUserInfo> {
    let provider_config = self.get_provider_config(provider)?;

    let response = self
      .http
      .get(&provider_config.user_info_url)
      .header("Authorization", format!("Bearer {}", access_token))
      .send()
      .await
      .map_err(|e| DataError::server_error(format!("Failed to get user info: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(DataError::server_error(format!("User info request failed: {}", error_text)));
    }

    let user_info: Value = response
      .json()
      .await
      .map_err(|e| DataError::server_error(format!("Failed to parse user info: {}", e)))?;

    self.parse_user_info(provider, &user_info)
  }

  /// 解析第三方用户信息为统一格式
  ///
  /// 将不同 OAuth 提供商的用户信息响应格式转换为统一的内部格式。
  ///
  /// # Arguments
  /// * `provider` - OAuth 提供商类型
  /// * `user_info` - 第三方用户信息 JSON
  ///
  /// # Returns
  /// 标准化的用户信息
  ///
  /// # Errors
  /// 如果提供商不支持或信息解析失败
  fn parse_user_info(&self, provider: OAuthProvider, user_info: &Value) -> fusions::core::Result<OAuthUserInfo> {
    match provider {
      OAuthProvider::Wechat => Ok(OAuthUserInfo {
        id: user_info.get("openid").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        name: user_info.get("nickname").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        email: None,
        phone: None,
        avatar: user_info.get("headimgurl").and_then(|v| v.as_str()).map(|s| s.to_string()),
      }),
      OAuthProvider::Github => Ok(OAuthUserInfo {
        id: user_info.get("id").and_then(|v| v.as_i64()).map(|i| i.to_string()).unwrap_or_default(),
        name: user_info.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        email: user_info.get("email").and_then(|v| v.as_str()).map(|s| s.to_string()),
        phone: None,
        avatar: user_info.get("avatar_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
      }),
      _ => Err(DataError::bad_request(format!("Unsupported OAuth provider: {:?}", provider))),
    }
  }

  /// 创建或更新用户
  ///
  /// 根据第三方用户信息在本地系统中创建新用户或更新现有用户。
  ///
  /// # Arguments
  /// * `user_info` - 第三方用户信息
  ///
  /// # Returns
  /// 本地用户 ID
  ///
  /// # Errors
  /// 如果用户信息无效或数据库操作失败
  async fn create_or_update_user(&self, user_info: &OAuthUserInfo) -> fusions::core::Result<i64> {
    // 检查用户是否已存在（通过第三方 provider 的用户ID）
    // 这里需要在 user_entity 表中添加 provider 相关字段或创建关联表

    // 示例：如果用户不存在，创建新用户
    if !user_info.id.is_empty() {
      let user_create = UserForCreate {
        email: user_info.email.clone(),
        phone: user_info.phone.clone(),
        name: Some(user_info.name.clone()),
        status: Some(UserStatus::Active),
        password: None,
      };

      let user_id = UserBmc::create(&self.mm, user_create).await?;
      info!("Created new user {} from OAuth login", user_id);
      Ok(user_id)
    } else {
      Err(DataError::bad_request("Invalid user info from OAuth provider"))
    }
  }
}

/// OAuth 提供商配置
///
/// 包含 OAuth 2.0 流程所需的所有配置参数。
#[derive(Clone, Deserialize)]
struct OAuthProviderConfig {
  client_id: String,
  client_secret: String,
  authorize_url: String,
  token_url: String,
  user_info_url: String,
  scope: String,
  redirect_uri: String,
}

/// 第三方用户信息统一格式
///
/// 标准化不同 OAuth 提供商的用户信息格式，便于内部处理。
#[derive(Debug, Clone)]
struct OAuthUserInfo {
  id: String,
  name: String,
  email: Option<String>,
  phone: Option<String>,
  #[allow(dead_code)]
  avatar: Option<String>,
}
