use rig::http_client::HttpClientExt;
use rig::image_generation::{ImageGenerationError, ImageGenerationRequest};
use rig::{http_client, image_generation};
use serde_json::json;

use crate::json_utils::merge_inplace;
use crate::providers::openai_compatible::{ApiResponse, Client};

// ================================================================
// OpenAI Image Generation API
// ================================================================

// 复用 rig 的常量
pub use rig::providers::openai::image_generation::{DALL_E_2, DALL_E_3, GPT_IMAGE_1};

// 复用 rig 的类型定义
pub use rig::providers::openai::image_generation::{ImageGenerationData, ImageGenerationResponse};

#[derive(Clone)]
pub struct ImageGenerationModel<T = reqwest::Client> {
  client: Client<T>,
  /// Name of the model (e.g.: dall-e-2)
  pub model: String,
}

impl<T> ImageGenerationModel<T> {
  pub(crate) fn new(client: Client<T>, model: &str) -> Self {
    Self { client, model: model.to_string() }
  }
}

impl<T> image_generation::ImageGenerationModel for ImageGenerationModel<T>
where
  T: HttpClientExt + Clone + Default + std::fmt::Debug + Send + 'static,
{
  type Response = ImageGenerationResponse;
  type Client = Client<T>;

  fn make(client: &Self::Client, model: impl Into<String>) -> Self {
    Self::new(client.clone(), &model.into())
  }

  #[cfg_attr(feature = "worker", worker::send)]
  async fn image_generation(
    &self,
    generation_request: ImageGenerationRequest,
  ) -> Result<image_generation::ImageGenerationResponse<Self::Response>, ImageGenerationError> {
    let mut request = json!({
        "model": self.model,
        "prompt": generation_request.prompt,
        "size": format!("{}x{}", generation_request.width, generation_request.height),
    });

    if self.model != *"gpt-image-1" {
      merge_inplace(
        &mut request,
        json!({
            "response_format": "b64_json"
        }),
      );
    }

    let body = serde_json::to_vec(&request)?;

    let request = self
      .client
      .post("/images/generations")?
      .header("Content-Type", "application/json")
      .body(body)
      .map_err(|e| ImageGenerationError::HttpError(e.into()))?;

    let response = self.client.send(request).await?;

    if !response.status().is_success() {
      let status = response.status();
      let text = http_client::text(response).await?;

      return Err(ImageGenerationError::ProviderError(format!("{}: {}", status, text,)));
    }

    let text = http_client::text(response).await?;

    match serde_json::from_str::<ApiResponse<ImageGenerationResponse>>(&text)? {
      ApiResponse::Ok(response) => response.try_into(),
      ApiResponse::Err(err) => Err(ImageGenerationError::ProviderError(err.message)),
    }
  }
}
