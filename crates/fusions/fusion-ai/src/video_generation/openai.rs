use reqwest::StatusCode;

use super::*;

const CREATE_URL: &str = "https://api.openai.com/v1/videos";
const STATUS_URL: &str = "https://api.openai.com/v1/videos";

#[derive(Debug, Deserialize)]
struct OpenAISubmitResp {
  pub id: String,
  #[allow(dead_code)]
  pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIStatusResp {
  pub status: String,
  pub output: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct OpenAIVideoProvider {
  client: reqwest::Client,
  api_key: String,
}

impl OpenAIVideoProvider {
  pub fn new(api_key: impl Into<String>) -> Self {
    Self { client: reqwest::Client::new(), api_key: api_key.into() }
  }
}

#[async_trait]
impl VideoGenerationProvider for OpenAIVideoProvider {
  async fn submit(&self, req: VideoGenerationRequest) -> Result<String, VideoGenerationError> {
    let resp = self
      .client
      .post(CREATE_URL)
      .bearer_auth(&self.api_key)
      .json(&serde_json::json!({
          "model": req.model,
          "prompt": req.prompt,
          "duration": req.duration,
          "size": req.size,
      }))
      .send()
      .await?;

    if resp.status() != StatusCode::OK && resp.status() != StatusCode::ACCEPTED {
      let body = resp.text().await.unwrap_or_default();
      return Err(VideoGenerationError::Api(format!("openai submit failed: {}", body)));
    }

    let parsed: OpenAISubmitResp = resp.json().await?;
    Ok(parsed.id)
  }

  async fn check_status(&self, request_id: &str) -> Result<VideoGenerationResponse, VideoGenerationError> {
    let url = format!("{}/{}", STATUS_URL, request_id);
    let resp = self.client.get(&url).bearer_auth(&self.api_key).send().await?;

    if resp.status() != StatusCode::OK {
      let body = resp.text().await.unwrap_or_default();
      return Err(VideoGenerationError::Api(format!("openai status failed: {}", body)));
    }

    let parsed: OpenAIStatusResp = resp.json().await?;
    let video_url = parsed.output.as_ref().and_then(|o| {
      // try find common fields
      if let Some(v) = o.get("video").and_then(|x| x.as_str()) {
        Some(v.to_string())
      } else if let Some(a) = o.get("data").and_then(|d| d.get(0)).and_then(|e| e.get("url")).and_then(|u| u.as_str()) {
        Some(a.to_string())
      } else {
        None
      }
    });

    Ok(VideoGenerationResponse {
      ready: parsed.status == "succeeded",
      video_url,
      meta: Some(serde_json::to_value(parsed).ok().unwrap_or(serde_json::Value::Null)),
    })
  }
}
