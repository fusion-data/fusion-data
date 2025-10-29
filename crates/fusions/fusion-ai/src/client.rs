use rig::agent::Agent;
use rig::client::ProviderClient;
use rig::client::builder::{BoxAgentBuilder, ClientBuildError, DynClientBuilder};
use rig::client::completion::CompletionModelHandle;
use rig::embeddings::embedding::EmbeddingModelDyn;

use crate::DefaultProviders;
use crate::agents::AgentConfig;
use crate::embeddings::EmbeddingConfig;
use crate::providers::openai_compatible::ClientWrapper;

pub struct ClientBuilderFactory {
  dyn_client_builder: DynClientBuilder,
}

impl Default for ClientBuilderFactory {
  fn default() -> Self {
    Self::new()
  }
}

impl ClientBuilderFactory {
  pub fn new() -> Self {
    let dyn_client_builder = DynClientBuilder::new();
    Self { dyn_client_builder }
  }

  pub fn client(
    &self,
    provider: &str,
    base_url: Option<&str>,
    api_key: Option<&str>,
  ) -> Result<Box<dyn ProviderClient>, ClientBuildError> {
    if provider == DefaultProviders::OPENAI_COMPATIBLE {
      if let Some(base_url) = base_url
        && let Some(api_key) = api_key
      {
        Ok(Box::new(ClientWrapper::new(base_url, api_key)))
      } else {
        Err(ClientBuildError::FactoryError("base_url or api_key".to_string()))
      }
    } else if let Some(api_key) = api_key {
      self.dyn_client_builder.build_val(provider, rig::client::ProviderValue::Simple(api_key.to_string()))
    } else {
      self.dyn_client_builder.build(provider)
    }
  }
}

impl ClientBuilderFactory {
  pub fn agent(&self, config: &AgentConfig) -> Result<Agent<CompletionModelHandle<'static>>, ClientBuildError> {
    let mut builder = self.agent_builder(config)?;

    if let Some(name) = config.name.as_deref().or(Some(config.provider.as_str())) {
      builder = builder.name(name);
    }

    if let Some(description) = config.description.as_deref() {
      builder = builder.description(description);
    }

    if let Some(system_prompt) = config.system_prompt.as_deref() {
      builder = builder.preamble(system_prompt);
    }

    for doc in config.static_context.iter() {
      builder = builder.context(doc);
    }

    if let Some(temperature) = config.temperature {
      builder = builder.temperature(temperature);
    }

    if let Some(max_tokens) = config.max_tokens {
      builder = builder.max_tokens(max_tokens);
    }

    if let Some(params) = config.additional_params.as_ref() {
      builder = builder.additional_params(params.clone());
    }

    // if let Some(tool_choice) = &config.tool_choice {
    //   builder = builder.tool_choice(tool_choice.clone());
    // }

    let agent = builder.build();

    Ok(agent)
  }

  pub fn agent_builder(&self, config: &AgentConfig) -> Result<BoxAgentBuilder<'static>, ClientBuildError> {
    let client = self.client(config.provider.as_str(), config.base_url.as_deref(), config.api_key.as_deref())?;

    let completion_client = client
      .as_completion()
      .ok_or(ClientBuildError::UnsupportedFeature(config.provider.clone(), "completion".to_string()))?;

    Ok(completion_client.agent(&config.model))
  }
}

impl ClientBuilderFactory {
  pub fn embeddings(&self, config: &EmbeddingConfig) -> Result<Box<dyn EmbeddingModelDyn>, ClientBuildError> {
    let client = self.client(config.provider.as_str(), config.base_url.as_deref(), config.api_key.as_deref())?;

    let embeddings = client
      .as_embeddings()
      .ok_or(ClientBuildError::UnsupportedFeature(config.provider.to_string(), "embeddings".to_owned()))?;

    Ok(embeddings.embedding_model_with_ndims(&config.model, config.dims))
  }
}
