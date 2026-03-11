use bytes::Bytes;

use rig::http_client::HttpClientExt;
use rig::http_client::multipart::{MultipartForm, Part};
use rig::transcription::{self, TranscriptionError};

use crate::providers::openai_compatible::{ApiResponse, Client};

// ================================================================
// OpenAI Transcription API
// ================================================================

// 复用 rig 的常量
pub use rig::providers::openai::transcription::WHISPER_1;

// 复用 rig 的类型定义
pub use rig::providers::openai::transcription::TranscriptionResponse;

#[derive(Clone)]
pub struct TranscriptionModel<T = reqwest::Client> {
  client: Client<T>,
  /// Name of the model (e.g.: gpt-3.5-turbo-1106)
  pub model: String,
}

impl<T> TranscriptionModel<T> {
  pub fn new(client: Client<T>, model: &str) -> Self {
    Self { client, model: model.to_string() }
  }
}

impl<T> transcription::TranscriptionModel for TranscriptionModel<T>
where
  T: HttpClientExt + Clone + std::fmt::Debug + Default + Send + 'static,
{
  type Response = TranscriptionResponse;
  type Client = Client<T>;

  fn make(client: &Self::Client, model: impl Into<String>) -> Self {
    Self::new(client.clone(), &model.into())
  }

  #[cfg_attr(feature = "worker", worker::send)]
  async fn transcription(
    &self,
    request: transcription::TranscriptionRequest,
  ) -> Result<transcription::TranscriptionResponse<Self::Response>, transcription::TranscriptionError> {
    let data = request.data;

    let mut body = MultipartForm::new()
      .text("model", self.model.clone())
      .part(Part::bytes("file", data).filename(request.filename));

    if let Some(language) = request.language {
      body = body.text("language", language);
    }

    if let Some(prompt) = request.prompt {
      body = body.text("prompt", prompt.clone());
    }

    if let Some(ref temperature) = request.temperature {
      body = body.text("temperature", temperature.to_string());
    }

    if let Some(ref additional_params) = request.additional_params {
      for (key, value) in additional_params
        .as_object()
        .expect("Additional Parameters to OpenAI Transcription should be a map")
      {
        body = body.text(key.to_owned(), value.to_string());
      }
    }

    let req = self
      .client
      .post("/audio/transcriptions")?
      .body(body)
      .map_err(|e| TranscriptionError::RequestError(Box::new(e)))?;

    let response = self.client.http_client.send_multipart::<Bytes>(req).await?;

    let status = response.status();
    let response_body = response.into_body().into_future().await?.to_vec();
    if status.is_success() {
      match serde_json::from_slice::<ApiResponse<TranscriptionResponse>>(&response_body)? {
        ApiResponse::Ok(response) => response.try_into(),
        ApiResponse::Err(api_error_response) => Err(TranscriptionError::ProviderError(api_error_response.message)),
      }
    } else {
      let str = String::from_utf8_lossy(&response_body).to_string();
      Err(TranscriptionError::ProviderError(str))
    }
  }
}
