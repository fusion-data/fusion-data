use bytes::Bytes;
use reqwest::multipart::Part;
use rig::http_client::HttpClientExt;
use rig::image_generation::ImageGenerationError;
use serde::{Deserialize, Serialize};

use crate::providers::openai_compatible::{ApiResponse, Client};

// Re-use from image_generation module
use super::image_generation::ImageGenerationData;

// ================================================================
// OpenAI Image Edit API
// ================================================================

// Model constants are exported from image_generation module
pub use super::image_generation::{DALL_E_2, GPT_IMAGE_1};

/// Request for image editing
#[derive(Clone, Debug)]
pub struct ImageEditRequest {
  /// The images to edit. For DALL-E 2: exactly 1 PNG image.
  /// For gpt-image-1: 1-16 images in PNG/JPG/WEBP format.
  pub images: Vec<Vec<u8>>,
  /// Optional mask image. Must be a valid PNG file with same dimensions as the first image.
  /// The fully transparent areas (alpha = 0) indicate where the image should be edited.
  /// Only supported for DALL-E 2 when using a single image.
  pub mask_data: Option<Vec<u8>>,
  /// A text description of the desired image(s).
  pub prompt: String,
  /// The size of the generated images in pixels.
  /// For gpt-image-1: "1024x1024", "1536x1024", "1024x1536", or "auto"
  /// For DALL-E 2: "256x256", "512x512", or "1024x1024"
  pub size: String,
  /// The number of images to generate (1-10). Defaults to 1.
  pub n: Option<u64>,
  /// The unique identifier for the end-user.
  pub user: Option<String>,
  /// The quality of the image that will be generated (gpt-image-1 only).
  /// Options: "low", "medium", "high". Defaults to "auto".
  pub quality: Option<String>,
  /// Allows setting transparency for the background of the generated image(s) (gpt-image-1 only).
  /// Options: "transparent", "opaque", "auto".
  pub background: Option<String>,
  /// The format in which the generated images are returned (gpt-image-1 only).
  /// Options: "png", "jpeg", "webp". Defaults to "png".
  pub output_format: Option<String>,
  /// The compression level for the generated images, 0-100 (gpt-image-1 with webp/jpeg only).
  pub output_compression: Option<u64>,
  /// Control how much effort the model will exert to match the style (gpt-image-1 only).
  /// Options: "high", "low". Defaults to "low".
  pub input_fidelity: Option<String>,
  /// The number of partial images to generate for streaming (gpt-image-1 only).
  /// Value must be 0-3. When 0, returns a single image.
  pub partial_images: Option<u64>,
  /// Whether to edit the image in streaming mode (gpt-image-1 only).
  pub stream: Option<bool>,
}

impl ImageEditRequest {
  /// Create a new basic image edit request with one image
  pub fn new(image_data: Vec<u8>, prompt: String, size: String) -> Self {
    Self {
      images: vec![image_data],
      mask_data: None,
      prompt,
      size,
      n: Some(1),
      user: None,
      quality: None,
      background: None,
      output_format: None,
      output_compression: None,
      input_fidelity: None,
      partial_images: None,
      stream: None,
    }
  }

  /// Create a new image edit request with multiple images (gpt-image-1 only)
  pub fn new_with_images(images: Vec<Vec<u8>>, prompt: String, size: String) -> Self {
    Self {
      images,
      mask_data: None,
      prompt,
      size,
      n: Some(1),
      user: None,
      quality: None,
      background: None,
      output_format: None,
      output_compression: None,
      input_fidelity: None,
      partial_images: None,
      stream: None,
    }
  }

  /// Set the mask data (for DALL-E 2 single image editing)
  pub fn with_mask(mut self, mask_data: Vec<u8>) -> Self {
    self.mask_data = Some(mask_data);
    self
  }

  /// Set the number of images to generate
  pub fn with_n(mut self, n: u64) -> Self {
    self.n = Some(n);
    self
  }

  /// Set the user identifier
  pub fn with_user(mut self, user: String) -> Self {
    self.user = Some(user);
    self
  }

  /// Set the quality (gpt-image-1 only)
  pub fn with_quality(mut self, quality: String) -> Self {
    self.quality = Some(quality);
    self
  }

  /// Set the background mode (gpt-image-1 only)
  pub fn with_background(mut self, background: String) -> Self {
    self.background = Some(background);
    self
  }

  /// Set the output format (gpt-image-1 only)
  pub fn with_output_format(mut self, format: String) -> Self {
    self.output_format = Some(format);
    self
  }

  /// Set the output compression (gpt-image-1 only)
  pub fn with_output_compression(mut self, compression: u64) -> Self {
    self.output_compression = Some(compression);
    self
  }

  /// Set the input fidelity (gpt-image-1 only)
  pub fn with_input_fidelity(mut self, fidelity: String) -> Self {
    self.input_fidelity = Some(fidelity);
    self
  }

  /// Set streaming mode (gpt-image-1 only)
  pub fn with_stream(mut self, stream: bool) -> Self {
    self.stream = Some(stream);
    self
  }
}

/// Token usage information for image generation (gpt-image-1 only)
#[derive(Debug, Deserialize, Serialize)]
pub struct Usage {
  pub total_tokens: i64,
  pub input_tokens: i64,
  pub output_tokens: i64,
  #[serde(default)]
  pub input_tokens_details: InputTokensDetails,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct InputTokensDetails {
  pub text_tokens: i64,
  pub image_tokens: i64,
}

/// Response from image edit API
#[derive(Debug, Deserialize)]
pub struct ImageEditResponse {
  pub created: i64,
  pub data: Vec<ImageGenerationData>,
  /// The background parameter used for the image generation (gpt-image-1 only)
  #[serde(default)]
  pub background: Option<String>,
  /// The output format of the image generation (gpt-image-1 only)
  #[serde(default)]
  pub output_format: Option<String>,
  /// The quality of the image generated (gpt-image-1 only)
  #[serde(default)]
  pub quality: Option<String>,
  /// The size of the image generated
  #[serde(default)]
  pub size: Option<String>,
  /// Token usage information (gpt-image-1 only)
  #[serde(default)]
  pub usage: Option<Usage>,
}

impl TryFrom<ImageEditResponse> for rig::image_generation::ImageGenerationResponse<ImageEditResponse> {
  type Error = ImageGenerationError;

  fn try_from(value: ImageEditResponse) -> Result<Self, Self::Error> {
    let url = value.data[0].url.as_str();
    let bytes = if url.is_empty() {
      // Decode from base64
      base64::Engine::decode(&base64::prelude::BASE64_STANDARD, &value.data[0].b64_json)
        .map_err(|e| ImageGenerationError::ResponseError(e.to_string()))?
    } else {
      // Download from URL
      log::info!("Download image from URL: {}", url);
      ureq::get(url)
        .call()
        .map_err(|e| ImageGenerationError::ResponseError(e.to_string()))?
        .into_body()
        .read_to_vec()
        .map_err(|e| ImageGenerationError::ResponseError(e.to_string()))?
    };

    Ok(rig::image_generation::ImageGenerationResponse { image: bytes, response: value })
  }
}

// ================================================================
// Image Edit Model
// ================================================================

#[derive(Clone)]
pub struct ImageEditModel<T = reqwest::Client> {
  client: Client<T>,
  /// Name of the model (e.g.: qwen-image-edit, gpt-image-1, dall-e-2)
  pub model: String,
}

impl<T> ImageEditModel<T> {
  pub(crate) fn new(client: Client<T>, model: &str) -> Self {
    Self { client, model: model.to_string() }
  }

  /// Build multipart form from request
  ///
  /// Note: We add multiple images using the same field name "image" multiple times.
  /// This is the standard way to send arrays in multipart/form-data and is compatible with:
  /// - OpenAI API (gpt-image-1, which supports up to 16 images)
  /// - Gitee AI API (various models, typically single image)
  /// - DALL-E 2 (single image only)
  fn build_form(&self, request: &ImageEditRequest) -> Result<reqwest::multipart::Form, ImageGenerationError> {
    let mut body = reqwest::multipart::Form::new()
      .text("model", self.model.clone())
      .text("prompt", request.prompt.clone())
      .text("size", request.size.clone());

    // Add all images using the same field name "image" multiple times
    // This creates an array in multipart/form-data which is compatible with OpenAI and Gitee AI
    for (idx, image_data) in request.images.iter().enumerate() {
      let file_name = if request.images.len() == 1 { "image.png".to_string() } else { format!("image_{}.png", idx) };

      body = body.part(
        "image",
        Part::bytes(image_data.clone())
          .file_name(file_name)
          .mime_str("image/png")
          .map_err(|e| ImageGenerationError::RequestError(format!("Invalid mime type: {}", e).into()))?,
      );
    }

    // Add optional mask (only for DALL-E 2 single image)
    if let Some(mask_data) = &request.mask_data {
      body = body.part(
        "mask",
        Part::bytes(mask_data.clone())
          .file_name("mask.png")
          .mime_str("image/png")
          .map_err(|e| ImageGenerationError::RequestError(format!("Invalid mime type: {}", e).into()))?,
      );
    }

    // Add common parameters
    if let Some(n) = request.n {
      body = body.text("n", n.to_string());
    }
    if let Some(user) = &request.user {
      body = body.text("user", user.clone());
    }

    // Add gpt-image-1 specific parameters
    if self.model == "gpt-image-1" {
      if let Some(quality) = &request.quality {
        body = body.text("quality", quality.clone());
      }
      if let Some(background) = &request.background {
        body = body.text("background", background.clone());
      }
      if let Some(format) = &request.output_format {
        body = body.text("output_format", format.clone());
      }
      if let Some(compression) = request.output_compression {
        body = body.text("output_compression", compression.to_string());
      }
      if let Some(fidelity) = &request.input_fidelity {
        body = body.text("input_fidelity", fidelity.clone());
      }
      if let Some(partial) = request.partial_images {
        body = body.text("partial_images", partial.to_string());
      }
      if let Some(stream) = request.stream {
        body = body.text("stream", stream.to_string());
      }
    } else {
      // DALL-E 2 specific: response_format (default to b64_json)
      body = body.text("response_format", "b64_json");
    }

    Ok(body)
  }
}

impl<T> ImageEditModel<T>
where
  T: HttpClientExt + Clone + std::fmt::Debug + Default + Send + 'static,
{
  /// Edit an image based on the provided request
  #[cfg_attr(feature = "worker", worker::send)]
  pub async fn image_edit(
    &self,
    request: ImageEditRequest,
  ) -> Result<rig::image_generation::ImageGenerationResponse<ImageEditResponse>, ImageGenerationError> {
    // Build the multipart form
    let body = self.build_form(&request)?;

    // Send request
    let req = self.client.post("/images/edits")?.body(body).unwrap();
    let response = self.client.http_client.send_multipart::<Bytes>(req).await?;

    let status = response.status();
    let response_body = response.into_body().into_future().await?.to_vec();

    if !status.is_success() {
      let text = String::from_utf8_lossy(&response_body).to_string();
      return Err(ImageGenerationError::ProviderError(format!("{}: {}", status, text)));
    }

    // Parse response
    match serde_json::from_slice::<ApiResponse<ImageEditResponse>>(&response_body)? {
      ApiResponse::Ok(response) => response.try_into(),
      ApiResponse::Err(err) => Err(ImageGenerationError::ProviderError(err.message)),
    }
  }

  /// Create a builder for image edit requests
  pub fn edit_request(&self, image_data: Vec<u8>, prompt: String, size: String) -> ImageEditRequest {
    ImageEditRequest::new(image_data, prompt, size)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_image_edit_request_builder() {
    let request = ImageEditRequest::new(vec![1, 2, 3, 4], "test prompt".to_string(), "1024x1024".to_string())
      .with_quality("high".to_string())
      .with_background("transparent".to_string())
      .with_n(2);

    assert_eq!(request.images[0], vec![1, 2, 3, 4]);
    assert_eq!(request.prompt, "test prompt");
    assert_eq!(request.size, "1024x1024");
    assert_eq!(request.quality, Some("high".to_string()));
    assert_eq!(request.background, Some("transparent".to_string()));
    assert_eq!(request.n, Some(2));
  }
}
