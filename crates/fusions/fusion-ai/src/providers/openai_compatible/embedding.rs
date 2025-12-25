use rig::embeddings::EmbeddingError;
use rig::http_client::HttpClientExt;
use rig::{embeddings, http_client};
use serde_json::json;

// 复用 rig 的常量
pub use rig::providers::openai::embedding::{TEXT_EMBEDDING_3_LARGE, TEXT_EMBEDDING_3_SMALL, TEXT_EMBEDDING_ADA_002};

// 复用 rig 的类型定义
pub use rig::providers::openai::embedding::{EmbeddingData, EmbeddingResponse};

use super::{ApiErrorResponse, ApiResponse, Client};

// ================================================================
// OpenAI Embedding API - 使用 rig 的 EmbeddingModel
// ================================================================

impl From<ApiErrorResponse> for EmbeddingError {
  fn from(err: ApiErrorResponse) -> Self {
    EmbeddingError::ProviderError(err.message)
  }
}

impl From<ApiResponse<EmbeddingResponse>> for Result<EmbeddingResponse, EmbeddingError> {
  fn from(value: ApiResponse<EmbeddingResponse>) -> Self {
    match value {
      ApiResponse::Ok(response) => Ok(response),
      ApiResponse::Err(err) => Err(EmbeddingError::ProviderError(err.message)),
    }
  }
}

#[derive(Clone)]
pub struct EmbeddingModel<T = reqwest::Client> {
  client: Client<T>,
  pub model: String,
  ndims: usize,
}

impl<T> embeddings::EmbeddingModel for EmbeddingModel<T>
where
  T: HttpClientExt + Clone + std::fmt::Debug + Default + Send + 'static,
{
  const MAX_DOCUMENTS: usize = 1024;

  type Client = Client<T>;

  fn make(client: &Self::Client, model: impl Into<String>, dims: Option<usize>) -> Self {
    let model_str = model.into();
    let ndims = dims.unwrap_or_else(|| match model_str.as_str() {
      TEXT_EMBEDDING_3_LARGE => 3072,
      TEXT_EMBEDDING_3_SMALL | TEXT_EMBEDDING_ADA_002 => 1536,
      _ => 0,
    });
    Self::new(client.clone(), &model_str, ndims)
  }

  fn ndims(&self) -> usize {
    self.ndims
  }

  #[cfg_attr(feature = "worker", worker::send)]
  async fn embed_texts(
    &self,
    documents: impl IntoIterator<Item = String>,
  ) -> Result<Vec<embeddings::Embedding>, EmbeddingError> {
    let documents = documents.into_iter().collect::<Vec<_>>();

    let mut body = json!({
        "model": self.model,
        "input": documents,
    });

    if self.ndims > 0 && self.model != TEXT_EMBEDDING_ADA_002 {
      body["dimensions"] = json!(self.ndims);
    }

    let body = serde_json::to_vec(&body)?;

    let req = self
      .client
      .post("/embeddings")?
      .header("Content-Type", "application/json")
      .body(body)
      .map_err(|e| EmbeddingError::HttpError(e.into()))?;

    let response = self.client.http_client.send(req).await?;

    if response.status().is_success() {
      let body: Vec<u8> = response.into_body().await?;
      let body: ApiResponse<EmbeddingResponse> = serde_json::from_slice(&body)?;

      match body {
        ApiResponse::Ok(response) => {
          tracing::info!(target: "rig",
              "OpenAI embedding token usage: {:?}",
              response.usage
          );

          if response.data.len() != documents.len() {
            return Err(EmbeddingError::ResponseError("Response data length does not match input length".into()));
          }

          Ok(
            response
              .data
              .into_iter()
              .zip(documents.into_iter())
              .map(|(embedding, document)| embeddings::Embedding { document, vec: embedding.embedding })
              .collect(),
          )
        }
        ApiResponse::Err(err) => Err(EmbeddingError::ProviderError(err.message)),
      }
    } else {
      let text = http_client::text(response).await?;
      Err(EmbeddingError::ProviderError(text))
    }
  }
}

impl<T> EmbeddingModel<T> {
  pub fn new(client: Client<T>, model: &str, ndims: usize) -> Self {
    Self { client, model: model.to_string(), ndims }
  }
}
