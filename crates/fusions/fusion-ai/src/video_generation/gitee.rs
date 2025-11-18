use reqwest::StatusCode;

use super::*;

// AI Gitee async generation endpoint
const SUBMIT_URL: &str = "https://ai.gitee.com/async/videos/generations";
const STATUS_URL: &str = "https://ai.gitee.com/async/videos/status";

#[derive(Debug, Serialize, Deserialize)]
struct GiteeSubmitResp {
  pub job_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GiteeStatusResp {
  pub status: String,
  #[serde(default)]
  pub video_url: Option<String>,
}

#[derive(Clone)]
pub struct GiteeVideoProvider {
  client: reqwest::Client,
  api_key: String,
}

impl GiteeVideoProvider {
  pub fn new(api_key: impl Into<String>) -> Self {
    Self { client: reqwest::Client::new(), api_key: api_key.into() }
  }
}

#[async_trait]
impl VideoGenerationProvider for GiteeVideoProvider {
  async fn submit(&self, req: VideoGenerationRequest) -> Result<String, VideoGenerationError> {
    let resp = self
      .client
      .post(SUBMIT_URL)
      .header("X-API-KEY", &self.api_key)
      .json(&serde_json::json!({
          "model": req.model,
          "prompt": req.prompt,
          "duration": req.duration,
          "size": req.size,
          "options": req.options,
      }))
      .send()
      .await?;

    if resp.status() != StatusCode::OK && resp.status() != StatusCode::ACCEPTED {
      let body = resp.text().await.unwrap_or_default();
      return Err(VideoGenerationError::Api(format!("gitee submit failed: {}", body)));
    }

    let parsed: GiteeSubmitResp = resp.json().await?;
    Ok(parsed.job_id)
  }

  async fn check_status(&self, request_id: &str) -> Result<VideoGenerationResponse, VideoGenerationError> {
    let url = format!("{}/{}", STATUS_URL, request_id);
    let resp = self.client.get(&url).header("X-API-KEY", &self.api_key).send().await?;

    if resp.status() != StatusCode::OK {
      let body = resp.text().await.unwrap_or_default();
      return Err(VideoGenerationError::Api(format!("gitee status failed: {}", body)));
    }

    let parsed: GiteeStatusResp = resp.json().await?;
    Ok(VideoGenerationResponse {
      ready: parsed.status == "succeeded",
      video_url: parsed.video_url.clone(),
      meta: Some(serde_json::to_value(parsed).ok().unwrap_or(serde_json::Value::Null)),
    })
  }
}
