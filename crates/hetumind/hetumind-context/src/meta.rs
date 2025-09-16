use http::header::HeaderMap;

pub static X_APP_VERSION: &str = "X-App-Version";
pub static X_DEVICE_ID: &str = "X-Device-Id";

#[derive(Clone, Default)]
pub struct RequestMetadata {
  app_version: String,
  device_id: String,
}

impl RequestMetadata {
  pub fn app_version(&self) -> &str {
    self.app_version.as_str()
  }

  pub fn device_id(&self) -> &str {
    self.device_id.as_str()
  }
}

impl From<&HeaderMap> for RequestMetadata {
  fn from(headers: &HeaderMap) -> Self {
    let app_version = headers.get(X_APP_VERSION).and_then(|v| v.to_str().ok()).unwrap_or_default().to_string();
    let device_id = headers.get(X_DEVICE_ID).and_then(|v| v.to_str().ok()).unwrap_or_default().to_string();
    Self { app_version, device_id }
  }
}
