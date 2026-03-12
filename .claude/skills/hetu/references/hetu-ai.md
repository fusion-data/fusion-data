# hetu-ai

AI 模块：LLM Providers、Agents、Graph Flow、Embeddings。

## Imports

```rust
use hetus::ai::{
    factory::{ClientFactory, AgentConfig, EmbeddingConfig, ProviderConfig},
    agents::{ModelAgent, AgentConfig},
    embeddings::{Embeddings, EmbeddingConfig},
    graph_flow::{
        Task, TaskResult, NextAction, Context,
        Graph, GraphBuilder, FlowRunner,
        Session, SessionStorage, InMemorySessionStorage,
        ExecutionResult, ExecutionStatus,
    },
};
use rig::completion::CompletionResponse;
```

## LLM Providers

### 支持的提供商

```rust
pub struct DefaultProviders;
impl DefaultProviders {
    pub const ANTHROPIC: &'static str = "anthropic";
    pub const OPENAI: &'static str = "openai";
    pub const OPENAI_COMPATIBLE: &'static str = "openai-compatible";
    pub const DEEPSEEK: &'static str = "deepseek";
    pub const GEMINI: &'static str = "gemini";
    pub const GROQ: &'static str = "groq";
    pub const MISTRAL: &'static str = "mistral";
    pub const MOONSHOT: &'static str = "moonshot";
    pub const OLLAMA: &'static str = "ollama";
    pub const AZURE: &'static str = "azure";
    // ... 更多
}
```

### ClientFactory

```rust
use hetus::ai::factory::ClientFactory;

let factory = ClientFactory::new();

// 创建客户端
let openai = factory.openai("sk-...")?;
let deepseek = factory.deepseek("sk-...", Some("https://api.deepseek.com"))?;
let ollama = factory.ollama("http://localhost:11434")?;
let anthropic = factory.anthropic("sk-ant-...")?;
```

## Agents

### AgentConfig

```rust
use hetus::ai::agents::AgentConfig;

let config = AgentConfig::builder()
    .provider("openai")
    .model("gpt-4")
    .api_key("sk-...")
    .system_prompt("You are a helpful assistant.")
    .temperature(0.7)
    .max_tokens(1000)
    .build()?;
```

### ModelAgent

```rust
use hetus::ai::agents::{ModelAgent, AgentConfig};
use rig::completion::CompletionResponse;

let agent = ModelAgent::new(config);

// 同步调用
let response: CompletionResponse<_> = agent.invoke("Hello!", vec![]).await?;
println!("{}", response.content());

// 流式调用
let stream = agent.stream("Hello!", vec![]).await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?);
}
```

## Graph Flow

### 定义 Task

```rust
use hetus::ai::graph_flow::{Task, TaskResult, NextAction, Context};
use async_trait::async_trait;

pub struct ProcessTask;

#[async_trait]
impl Task for ProcessTask {
    fn id(&self) -> &str {
        "process_task"
    }

    async fn run(&self, context: Context) -> Result<TaskResult, FlowError> {
        // 从 context 获取数据
        let input: String = context.get("input").await.unwrap_or_default();

        // 处理数据
        let output = format!("Processed: {}", input);

        // 存储结果
        context.set("output", output.clone()).await;

        // 返回结果
        Ok(TaskResult::new(
            Some(output),
            NextAction::Continue,
        ))
    }
}
```

### NextAction

```rust
pub enum NextAction {
    Continue,                    // 继续下一个任务
    ContinueAndExecute,          // 继续并立即执行
    WaitForInput,                // 等待用户输入
    End,                         // 结束流程
    GoTo(String),                // 跳转到指定任务
    Wait(NextTaskAndWaitFor),    // 等待指定条件
}
```

### 构建 Graph

```rust
use hetus::ai::graph_flow::{Graph, GraphBuilder};
use std::sync::Arc;

let graph = Arc::new(
    GraphBuilder::new("my_workflow")
        // 添加任务
        .add_task(Arc::new(StartTask))
        .add_task(Arc::new(ProcessTask))
        .add_task(Arc::new(EndTask))

        // 添加边
        .add_edge("start_task", "process_task")
        .add_edge("process_task", "end_task")

        // 条件边
        .add_conditional_edge(
            "process_task",
            |ctx| ctx.get_sync::<bool>("success").unwrap_or(false),
            "end_task",      // 条件为 true
            "start_task",    // 条件为 false（循环）
        )

        .build()
);
```

### FlowRunner

```rust
use hetus::ai::graph_flow::{FlowRunner, Session, InMemorySessionStorage};

// 创建存储
let storage = Arc::new(InMemorySessionStorage::new());

// 创建 runner
let runner = FlowRunner::new(graph, storage.clone());

// 创建 session
let session = Session::new_from_task(
    "session_123".to_string(),
    "start_task"
);
session.context.set("input", "Hello").await;
storage.save(session).await?;

// 执行 workflow
let result = runner.run("session_123").await?;

// 处理结果
match result.status {
    ExecutionStatus::Completed => println!("Done!"),
    ExecutionStatus::WaitingForInput => {
        println!("Waiting for input...");
        // 稍后继续
        runner.continue_with_input("session_123", "user input").await?;
    }
    ExecutionStatus::Paused { next_task_id, reason } => {
        println!("Paused at {}: {}", next_task_id, reason);
    }
    ExecutionStatus::Error(e) => eprintln!("Error: {}", e),
}
```

### Context

```rust
use hetus::ai::graph_flow::Context;

// 存储数据
context.set("key", "value").await;
context.set("count", 42).await;

// 获取数据
let value: Option<String> = context.get("key").await;
let count: i32 = context.get("count").await.unwrap_or(0);

// 同步获取
let value: Option<String> = context.get_sync("key");

// Chat history
context.add_user_message("Hello!").await;
context.add_assistant_message("Hi there!").await;
let messages = context.get_messages().await;
```

### Session Storage

```rust
use hetus::ai::graph_flow::{SessionStorage, InMemorySessionStorage, PostgresSessionStorage};

// 内存存储（开发/测试）
let storage = Arc::new(InMemorySessionStorage::new());

// PostgreSQL 存储（生产）
let storage = Arc::new(PostgresSessionStorage::new(pool));
```

## Embeddings

```rust
use hetus::ai::embeddings::{Embeddings, EmbeddingConfig};

let config = EmbeddingConfig::builder()
    .provider("openai")
    .model("text-embedding-3-small")
    .dims(1536)
    .api_key("sk-...")
    .build()?;

let embeddings = Embeddings::new(config);
let vectors = embeddings
    .embed(vec!["hello".to_string(), "world".to_string()])
    .await?;

// 返回 Vec<Vec<f32>>
for (text, vector) in texts.iter().zip(vectors.iter()) {
    println!("{}: {} dimensions", text, vector.len());
}
```

## Best Practices

1. **Provider 选择**: 使用 `ClientFactory` 统一创建客户端
2. **Graph Flow**: 对于交互式应用使用 `FlowRunner`，批处理直接使用 `Graph::execute`
3. **Context**: 使用 `context.set/get` 传递任务间数据
4. **Chat History**: 使用 `context.add_user_message/add_assistant_message` 管理 LLM 对话
5. **Session 持久化**: 生产环境使用 `PostgresSessionStorage`

## Examples from Codebase

- `crates/hetu-ai/src/factory/mod.rs` - ClientFactory 实现
- `crates/hetu-ai/src/graph_flow/mod.rs` - Graph Flow 核心
- `hetumind/hetumind-nodes/src/lm/lm_openai/mod.rs` - LLM 节点示例
