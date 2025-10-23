use axum::extract::FromRequestParts;
use fusion_core::application::Application;
use fusion_web::WebError;
use http::request::Parts;

/// OAuth 代理服务 - 简化版，只做重定向到 Jieyuan
#[derive(Clone)]
pub struct OAuthProxySvc {
  jieyuan_base_url: String,
}

impl OAuthProxySvc {
  pub fn new(jieyuan_base_url: String) -> Self {
    Self { jieyuan_base_url }
  }

  /// 获取 Jieyuan 登录页面 URL
  pub fn get_signin_url(&self, redirect_to: Option<&str>) -> String {
    let mut url = format!("{}/auth/signin", self.jieyuan_base_url);
    if let Some(redirect) = redirect_to {
      url.push_str(&format!("?redirect_to={}", urlencoding::encode(redirect)));
    }
    url
  }

  /// 获取 Jieyuan OAuth 授权 URL
  pub fn get_oauth_url(&self, provider: &str, redirect_to: Option<&str>) -> String {
    let mut url = format!("{}/auth/oauth/{}/start", self.jieyuan_base_url, provider);
    if let Some(redirect) = redirect_to {
      url.push_str(&format!("?redirect_to={}", urlencoding::encode(redirect)));
    }
    url
  }

  /// 获取 Jieyuan 注册页面 URL
  pub fn get_signup_url(&self, redirect_to: Option<&str>) -> String {
    let mut url = format!("{}/auth/signup", self.jieyuan_base_url);
    if let Some(redirect) = redirect_to {
      url.push_str(&format!("?redirect_to={}", urlencoding::encode(redirect)));
    }
    url
  }
}

impl FromRequestParts<Application> for OAuthProxySvc {
  type Rejection = WebError;

  async fn from_request_parts(_parts: &mut Parts, _state: &Application) -> Result<Self, Self::Rejection> {
    let jieyuan_base_url = std::env::var("JIEYUAN_BASE_URL").unwrap_or_else(|_| "http://localhost:50010".to_string());

    Ok(OAuthProxySvc::new(jieyuan_base_url))
  }
}
