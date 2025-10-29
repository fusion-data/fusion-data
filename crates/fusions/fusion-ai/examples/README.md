# Fusion AI Examples

This directory contains example code demonstrating how to use the fusion-ai library.

## OpenAI-Compatible Provider Example

### Overview

The `openai_compatible_example.rs` demonstrates how to use the OpenAI-compatible provider that works with various OpenAI API-compatible endpoints.

### Features

- **Multiple Provider Support**: Works with OpenAI, DeepSeek, Zhipu GLM, and other OpenAI-compatible APIs
- **Flexible Configuration**: Supports custom base URLs and API keys
- **Response Compatibility**: Handles responses with optional `object` field
- **Agent Integration**: Fully integrated with the ModelAgent system

### Usage

1. **Basic Setup with DynClientBuilder**:
   ```rust
   let mut dyn_builder = DynClientBuilder::new();
   let dyn_builder = register_openai_compatible_provider(dyn_builder);

   let client = dyn_builder.build_val(
     "openai-compatible",
     ProviderValue::Simple("your-api-key".to_string())
   )?;
   ```

2. **Using with AgentConfig**:
   ```rust
   let config = AgentConfig::builder()
     .provider("openai-compatible")
     .model("gpt-3.5-turbo")
     .api_key("your-api-key")
     .base_url("https://your-openai-compatible-endpoint.com/v1")
     .system_prompt("You are a helpful AI assistant.")
     .build()?;

   let agent = ModelAgent::new(config)?;
   ```

### Supported Providers

The OpenAI-compatible provider works with:
- OpenAI API
- DeepSeek API
- Zhipu GLM API
- Any other OpenAI API-compatible endpoints

### Important Notes

- The provider uses OpenAI's completion API format
- Response `object` field is handled as optional (some providers don't include it)
- Complex configuration (like base_url) is handled through AgentConfig
- The provider is automatically registered when creating ClientBuilderFactory

### Configuration

For providers that require special handling:
- **DeepSeek**: Does not support empty `tools` arrays
- **Zhipu GLM**: May lack the `object` field in responses

The provider automatically handles these cases by making the `object` field optional in response parsing.