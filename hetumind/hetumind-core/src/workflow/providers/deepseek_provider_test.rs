//! DeepSeek LLM Provider Tests
//!
//! Unit tests for the DeepSeek LLM SubNodeProvider implementation.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{
        sub_node_provider::{LLMConfig, Message},
        ClusterNodeConfig, ExecutionConfig,
    };

    #[test]
    fn test_deepseek_config_default() {
        let config = DeepSeekConfig::default();
        assert_eq!(config.model, "deepseek-chat");
        assert_eq!(config.max_tokens, Some(4096));
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.top_p, Some(1.0));
        assert_eq!(config.api_key, None);
    }

    #[test]
    fn test_llm_config_conversion() {
        let llm_config = LLMConfig {
            model: "deepseek-coder".to_string(),
            max_tokens: Some(8000),
            temperature: Some(0.5),
            top_p: Some(90),
            stop_sequences: Some(vec!["```".to_string()]),
            api_key: Some("test-key".to_string()),
        };

        let deepseek_config = DeepSeekConfig::from(llm_config);
        assert_eq!(deepseek_config.model, "deepseek-coder");
        assert_eq!(deepseek_config.max_tokens, Some(8000));
        assert_eq!(deepseek_config.temperature, Some(0.5));
        assert_eq!(deepseek_config.top_p, Some(90.0));
        assert_eq!(deepseek_config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_provider_creation() {
        let config = DeepSeekConfig {
            model: "deepseek-coder".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = DeepSeekLLMProvider::new(config);
        assert_eq!(provider.config().model, "deepseek-coder");
        assert_eq!(provider.config().api_key, Some("test-key".to_string()));
        assert!(!provider.provider_id().is_empty());
    }

    #[test]
    fn test_provider_from_llm_config() {
        let llm_config = LLMConfig {
            model: "deepseek-coder".to_string(),
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = DeepSeekLLMProvider::from_llm_config(llm_config);
        assert_eq!(provider.config().model, "deepseek-coder");
        assert_eq!(provider.config().api_key, Some("test-key".to_string()));
    }

    #[tokio::test]
    async fn test_provider_initialization() {
        let config = DeepSeekConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };

        let provider = DeepSeekLLMProvider::new(config);
        let result = provider.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_provider_initialization_without_api_key() {
        let config = DeepSeekConfig {
            api_key: None,
            ..Default::default()
        };

        let provider = DeepSeekLLMProvider::new(config);
        let result = provider.initialize().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_provider_initialization_with_env_var() {
        // Set environment variable
        std::env::set_var("DEEPSEEK_API_KEY", "env-test-key");

        let config = DeepSeekConfig {
            api_key: None,
            ..Default::default()
        };

        let provider = DeepSeekLLMProvider::new(config);
        let result = provider.initialize().await;
        assert!(result.is_ok());

        // Clean up
        std::env::remove_var("DEEPSEEK_API_KEY");
    }

    #[tokio::test]
    async fn test_llm_call() {
        let config = DeepSeekConfig {
            api_key: Some("test-key".to_string()),
            model: "deepseek-chat".to_string(),
            ..Default::default()
        };

        let provider = DeepSeekLLMProvider::new(config);

        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Hello, DeepSeek!".to_string(),
            }
        ];

        let llm_config = LLMConfig {
            model: "deepseek-chat".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            ..Default::default()
        };

        let result = provider.call_llm(messages, llm_config).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.content.is_empty());
        assert_eq!(response.role, "assistant");
        assert!(response.usage.is_some());
    }

    #[tokio::test]
    async fn test_provider_factory() {
        let config = Some(DeepSeekConfig {
            model: "deepseek-coder".to_string(),
            api_key: Some("factory-key".to_string()),
            ..Default::default()
        });

        let provider = create_deepseek_provider(config);
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.config().model, "deepseek-coder");
        assert_eq!(provider.config().api_key, Some("factory-key".to_string()));
    }

    #[test]
    fn test_node_definition() {
        let config = DeepSeekConfig {
            model: "deepseek-coder".to_string(),
            ..Default::default()
        };

        let provider = DeepSeekLLMProvider::new(config);
        let node_def = provider.get_node_definition();

        assert_eq!(node_def.kind, "deepseek_llm");
        assert!(node_def.display_name.contains("DeepSeek LLM Provider"));
        assert!(node_def.description.is_some());
        assert!(node_def.document_url.is_some());
        assert_eq!(node_def.document_url.unwrap(), "https://platform.deepseek.com/");
    }

    #[test]
    fn test_provider_type() {
        let config = DeepSeekConfig::default();
        let provider = DeepSeekLLMProvider::new(config);

        let provider_type = provider.provider_type();
        assert!(matches!(provider_type, SubNodeProviderType::LLM));
    }

    #[test]
    fn test_config_update() {
        let mut provider = DeepSeekLLMProvider::new(DeepSeekConfig::default());

        let new_config = DeepSeekConfig {
            model: "deepseek-coder".to_string(),
            max_tokens: Some(8000),
            api_key: Some("new-key".to_string()),
            ..Default::default()
        };

        provider.update_config(new_config);
        assert_eq!(provider.config().model, "deepseek-coder");
        assert_eq!(provider.config().max_tokens, Some(8000));
        assert_eq!(provider.config().api_key, Some("new-key".to_string()));
    }
}