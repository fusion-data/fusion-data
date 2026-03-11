# hetu-ai LLM 使用指南

> hetu-ai 是 Fusion-Data 项目的 AI 集成库，基于 rig-core，提供多 LLM 提供商支持、Agent、Embeddings、Graph Flow 工作流等功能。

## 架构概览

```
hetu-ai/
├── agents/          # Agent 封装
├── client.rs        # ClientFactory 工厂模式
├── embeddings.rs    # Embedding 配置
├── error.rs         # 错误类型
├── factory/         # 工厂模块 (推荐 v0.27+)
├── graph_flow/      # 工作流引擎
│   ├── context.rs   # 上下文管理
│   ├── graph.rs     # 图构建器
│   ├── runner.rs    # 执行器
│   ├── storage.rs   # 存储后端
│   └── task.rs      # 任务定义
├── providers/       # LLM 提供商
│   └── openai_compatible/  # OpenAI 兼容接口
├── json_utils.rs    # JSON 工具
└── utils.rs         # 通用工具
```

---

## ClientFactory 工厂模式 (推荐)

### 创建客户端

```rust
use hetu_ai::factory::ClientFactory;

let factory = ClientFactory::new();

// OpenAI
let openai_client = factory.openai("sk-...")?;

// Anthropic
let anthropic_client = factory.anthropic("sk-...")?;

// DeepSeek (支持自定义 base_url)
let deepseek_client = factory.deepseek("sk-...", Some("https://api.deepseek.com"))?;

// Groq
let groq_client = factory.groq("gsk-...")?;

// xAI
let xai_client = factory.xai("sk-...", None)?;

// OpenRouter
let openrouter_client = factory.openrouter("sk-...")?;

// Google/Gemini
let google_client = factory.google("AIza...")?;

// Cohere
let cohere_client = factory.cohere("Coherent-...")?;

// HuggingFace
let hf_client = factory.huggingface("hf-...")?;

// TogetherAI
let together_client = factory.togetherai("tog-...")?;

// Perplexity
let perplexity_client = factory.perplexity("pplx-...")?;

// Mistral
let mistral_client = factory.mistral("mistral-...")?;

// Ollama (本地)
let ollama_client = factory.ollama("http://localhost:11434")?;

// OpenAI 兼容客户端
let compatible_client = factory.openai_compatible("https://api.example.com", "sk-...");
```

---

## AgentConfig

```rust
use hetu_ai::factory::{AgentConfig, ClientFactory};

let config = AgentConfig::builder()
  .provider("openai")
  .model("gpt-4o")
  .system_prompt("You are a helpful assistant.")
  .temperature(0.7)
  .max_tokens(1024)
  .build();

// 或使用结构体
let config = AgentConfig {
  provider: "openai".into(),
  model: "gpt-4o".into(),
  base_url: None,
  api_key: None,
  name: None,
  description: None,
  system_prompt: Some("You are helpful.".into()),
  static_context: vec![],
  static_tools: vec![],
  max_tokens: Some(1024),
  temperature: Some(0.7),
  additional_params: None,
};
```

### 创建 Agent

```rust
use hetu_ai::factory::{AgentConfig, ClientFactory};

let factory = ClientFactory::new();
let client = factory.openai("sk-...")?;

let agent = factory.openai_agent(&config, &client)?;
```

### Agent 调用

```rust
// 非流式
let response = agent.completion("Hello!").await?;
println!("Response: {}", response.choices[0].message.content.clone().unwrap_or_default());

// 流式
let mut stream = agent.completion("Hello!").await?.stream().await?;
while let Some(chunk) = stream.next().await {
  let content = chunk?.choices[0].delta.content.clone();
  print!("{}", content.unwrap_or_default());
}
```

---

## ModelAgent 简单封装

```rust
use hetu_ai::agents::AgentConfig;

let config = AgentConfig::builder()
  .provider("openai")
  .model("gpt-4o")
  .system_prompt("You are a helpful assistant.")
  .build();

let agent = ModelAgent::<OpenAICompletionModel>::new(config);

// 调用
let response = agent.invoke("Hello!").await?;
let stream = agent.stream("Hello!").await?;
```

---

## Embeddings 嵌入

### EmbeddingConfig

```rust
use hetu_ai::factory::{EmbeddingConfig, ClientFactory};

let config = EmbeddingConfig::builder()
  .provider("openai")
  .model("text-embedding-3-small")
  .dims(1536)
  .build();

let factory = ClientFactory::new();
let embeddings = factory.embeddings(&config, vec!["Hello world".into()]).await?;
```

### Embeddings 封装

```rust
use hetu_ai::embeddings::{EmbeddingConfig, Embeddings};

let config = EmbeddingConfig::builder()
  .provider("openai")
  .model("text-embedding-3-small")
  .dims(1536)
  .build();

let embeddings = Embeddings::new(config);
let result = embeddings.embed(vec!["Hello world".into()]).await?;
```

---

## Graph Flow 工作流

### Task Trait

```rust
use hetu_ai::graph_flow::{Task, TaskResult, NextAction, Context};
use async_trait::async_trait;

struct HelloTask;

#[async_trait]
impl Task for HelloTask {
  fn id(&self) -> &str {
    "hello_task"
  }

  async fn run(&self, context: Context) -> hetu_ai::graph_flow::Result<TaskResult> {
    let name: String = context.get("name").await.unwrap_or("World".to_string());
    let greeting = format!("Hello, {}!", name);

    context.set("greeting", greeting.clone()).await;
    Ok(TaskResult::new(Some(greeting), NextAction::Continue))
  }
}
```

### Context

```rust
use hetu_ai::graph_flow::Context;

let context = Context::new();

// 存储/检索
context.set("key", "value").await;
let value: Option<String> = context.get("key").await;

// 聊天历史
context.add_user_message("Hello!".into()).await;
context.add_assistant_message("Hi there!".into()).await;

// 消息历史
let history = context.chat_history().await;
```

### GraphBuilder

```rust
use hetu_ai::graph_flow::{GraphBuilder, Task, NextAction};
use std::sync::Arc;

let hello_task = Arc::new(HelloTask);

let graph = GraphBuilder::new("greeting_workflow")
  .add_task(hello_task.clone())
  .build();

// 添加边 (顺序执行)
graph.add_edge(task1.id(), task2.id());

// 添加条件边
graph.add_conditional_edge(
  task2.id(),
  |ctx| ctx.get_sync::<bool>("condition").unwrap_or(false),
  task3.id(),    // if true
  task1.id(),    // if false
);
```

### FlowRunner

```rust
use hetu_ai::graph_flow::{FlowRunner, InMemorySessionStorage, Session};

let storage = Arc::new(InMemorySessionStorage::new());
let runner = FlowRunner::new(graph.clone(), storage.clone());

// 创建会话
let session = Session::new_from_task("user_123".into(), "hello_task");
session.context.set("name", "Alice".into()).await;
storage.save(session).await?;

// 执行工作流
let result = runner.run("user_123").await?;
println!("Response: {:?}", result.response);
```

### 存储后端

```rust
// 内存存储 (开发/测试)
use hetu_ai::graph_flow::InMemorySessionStorage;
let storage = Arc::new(InMemorySessionStorage::new());

// PostgreSQL 存储 (生产)
use hetu_ai::graph_flow::PostgresSessionStorage;
let storage = Arc::new(PostgresSessionStorage::new(pool));
```

---

## ProviderClientEnum

```rust
pub enum ProviderClientEnum {
  OpenAI(Result<openai::Client, http_client::Error>),
  Anthropic(Result<anthropic::Client, http_client::Error>),
  DeepSeek(Result<deepseek::Client, http_client::Error>),
  Groq(Result<groq::Client, http_client::Error>),
  XAI(Result<xai::Client, http_client::Error>),
  OpenRouter(Result<openrouter::Client, http_client::Error>),
  Google(Result<gemini::Client, http_client::Error>),
  Cohere(Result<cohere::Client, http_client::Error>),
  HuggingFace(Result<huggingface::Client, http_client::Error>),
  TogetherAI(Result<together::Client, http_client::Error>),
  Perplexity(Result<perplexity::Client, http_client::Error>),
  Mistral(Result<mistral::Client, http_client::Error>),
  Ollama(Result<ollama::Client, http_client::Error>),
  OpenAICompatible(ClientWrapper),
}
```

---

## 默认提供商常量

```rust
hetu_ai::DefaultProviders::ANTHROPIC        // "anthropic"
hetu_ai::DefaultProviders::COHERE           // "cohere"
hetu_ai::DefaultProviders::GEMINI           // "gemini"
hetu_ai::DefaultProviders::HUGGINGFACE      // "huggingface"
hetu_ai::DefaultProviders::OPENAI           // "openai"
hetu_ai::DefaultProviders::OPENAI_COMPATIBLE // "openai-compatible"
hetu_ai::DefaultProviders::OPENROUTER       // "openrouter"
hetu_ai::DefaultProviders::TOGETHER         // "together"
hetu_ai::DefaultProviders::XAI              // "xai"
hetu_ai::DefaultProviders::AZURE            // "azure"
hetu_ai::DefaultProviders::DEEPSEEK         // "deepseek"
hetu_ai::DefaultProviders::MISTRAL          // "mistral"
hetu_ai::DefaultProviders::OLLAMA           // "ollama"
hetu_ai::DefaultProviders::PERPLEXITY       // "perplexity"
```

---

## 使用模式

### 完整示例

```rust
use hetu_ai::factory::{AgentConfig, ClientFactory};
use rig::message::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let factory = ClientFactory::new();
  let client = factory.openai("sk-...")?;

  let config = AgentConfig::builder()
    .provider("openai")
    .model("gpt-4o")
    .system_prompt("You are a helpful assistant.")
    .temperature(0.7)
    .build();

  let agent = factory.openai_agent(&config, &client)?;

  let messages = vec![
    Message::user("What is Rust?"),
  ];

  let response = agent.completion("What is Rust?", messages).await?;
  println!("Response: {}", response.choices[0].message.content.clone().unwrap_or_default());

  Ok(())
}
```

---

**版本**: hetu-ai v0.1.0+
