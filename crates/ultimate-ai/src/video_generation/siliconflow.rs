use reqwest::StatusCode;

use super::*;

// endpoints aligned with docs you provided
const SUBMIT_URL: &str = "https://api.siliconflow.cn/v1/videos/submit";
const STATUS_URL: &str = "https://api.siliconflow.cn/v1/videos/status";

#[derive(Debug, Deserialize)]
struct SiliconFlowSubmitResp {
  pub request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SiliconFlowStatusResp {
  pub status: String,
  #[serde(default)]
  pub video_url: Option<String>,
}

#[derive(Clone)]
pub struct SiliconFlowVideoProvider {
  client: reqwest::Client,
  api_key: String,
}

impl SiliconFlowVideoProvider {
  pub fn new(api_key: impl Into<String>) -> Self {
    Self { client: reqwest::Client::new(), api_key: api_key.into() }
  }
}

#[async_trait]
impl VideoGenerationProvider for SiliconFlowVideoProvider {
  async fn submit(&self, req: VideoGenerationRequest) -> Result<String, VideoGenerationError> {
    let resp = self
      .client
      .post(SUBMIT_URL)
      .bearer_auth(&self.api_key)
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
      return Err(VideoGenerationError::Api(format!("siliconflow submit failed: {}", body)));
    }

    let parsed: SiliconFlowSubmitResp = resp.json().await?;
    Ok(parsed.request_id)
  }

  async fn check_status(&self, request_id: &str) -> Result<VideoGenerationResponse, VideoGenerationError> {
    let url = format!("{}/{}", STATUS_URL, request_id);
    let resp = self.client.get(&url).bearer_auth(&self.api_key).send().await?;

    if resp.status() != StatusCode::OK {
      let body = resp.text().await.unwrap_or_default();
      return Err(VideoGenerationError::Api(format!("siliconflow status failed: {}", body)));
    }

    let parsed: SiliconFlowStatusResp = resp.json().await?;
    Ok(VideoGenerationResponse {
      ready: parsed.status == "succeeded",
      video_url: parsed.video_url.clone(),
      meta: Some(serde_json::to_value(parsed).ok().unwrap_or(serde_json::Value::Null)),
    })
  }
}
