# OpenAI-Compatible Provider

这个提供商专门为使用 completion API（而不是 responses API）的 OpenAI 兼容 API 设计，确保与大多数模型提供商（如 DeepSeek、智谱 GLM 等）的兼容性。

## 特性

- ✅ **默认使用 Completion API**：使用传统的 `/chat/completions` API，而不是 OpenAI 最新的 responses API
- ✅ **多提供商兼容**：支持 DeepSeek、智谱 GLM、月之暗面等所有 OpenAI 兼容的 API
- ✅ **DynClientBuilder 集成**：完全集成到 rig 的 DynClientBuilder 模式中
- ✅ **自定义 Base URL 支持**：可以指定自定义的 API 端点
- ✅ **响应兼容性**：处理缺少 `object` 字段的响应（如智谱 GLM）

## 架构设计

### Client 结构

```rust
pub struct Client<T = reqwest::Client> {
  base_url: String,
  api_key: String,
  http_client: T,
}
```

- `Client` 是一个包装器，包含 base URL、API key 和 HTTP client
- 通过 `to_openai_client()` 方法转换为标准的 OpenAI client
- 实现了所有必需的 provider traits

### Completion API 集成

```rust
impl CompletionClient for Client<reqwest::Client> {
  type CompletionModel = CompletionModel<reqwest::Client>;

  fn completion_model(&self, model: &str) -> Self::CompletionModel {
    let openai_client = self.to_openai_client();
    CompletionModel::new(openai_client, model)
  }
}
```

- 使用 `rig::providers::openai::completion::CompletionModel`
- 确保使用 completion API 而不是 responses API

## 使用方式

### 1. 通过 DynClientBuilder

```rust
let dyn_builder = register_openai_compatible_provider(DynClientBuilder::new());

let client = dyn_builder.build_val(
  "openai-compatible",
  ProviderValue::Simple("your-api-key".to_string())
)?;
```

### 2. 通过 AgentConfig

```rust
let config = AgentConfigBuilder::default()
  .provider("openai-compatible")
  .model("deepseek-chat")
  .api_key("your-api-key")
  .base_url("https://api.deepseek.com/v1")
  .system_prompt("你是一个 AI 助手")
  .temperature(0.7)
  .build()?;

let agent = ModelAgent::try_from(&config)?;
```

## 支持的功能

- ✅ **Completion API**: 完整支持文本补全和对话
- ❌ **Embeddings**: 不支持（返回 ProviderError）
- ❌ **Transcription**: 不支持（返回 ProviderError）
- ❌ **Image Generation**: 不支持
- ❌ **Audio Generation**: 不支持

## 注意事项

### DeepSeek 特殊处理
- 不支持传递空数组的 `tools`（即：`tools: []`），会报错
- 建议在使用时不传递 tools 参数

### 智谱 GLM 特殊处理
- 响应中缺少 `object` 字段，已通过使用 completion API 避免
- 建议使用 completion API 而不是 responses API

## 注册方式

提供商在 `ClientBuilderFactory::new()` 时自动注册：

```rust
impl ClientBuilderFactory {
  pub fn new() -> Self {
    let dyn_client_builder = register_openai_compatible_provider(DynClientBuilder::new());
    Self { dyn_client_builder }
  }
}
```

## 设计原理

1. **兼容性优先**：选择 completion API 而不是 responses API，因为大部分模型提供商只支持前者
2. **零配置集成**：通过 DynClientBuilder 自动注册，无需额外配置
3. **错误处理**：不支持的功能返回明确的错误信息，而不是静默失败
4. **类型安全**：充分利用 Rust 的类型系统确保编译时安全

这个实现确保了与各种 OpenAI 兼容 API 的最大兼容性，同时保持了与 rig 生态系统的完整集成。