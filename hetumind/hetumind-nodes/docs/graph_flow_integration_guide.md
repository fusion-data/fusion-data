# Graph-flow Integration Guide

## 概述

本文档描述了如何使用基于 `graph-flow` 框架重构的 AI Agent、Memory 和 LLM 节点。这次重构将复杂的 AI 工作流分解为一系列可组合的任务，提供了更好的模块化、可测试性和可扩展性。

## 架构设计

### 核心概念

1. **任务驱动架构**：每个功能都被实现为 `graph-flow::Task` trait
2. **内存存储**：使用 `InMemorySessionStorage` 和 `InMemoryGraphStorage`
3. **工作流编排**：通过 `FlowRunner` 协调任务执行
4. **会话管理**：支持会话隔离和状态管理

### 模块结构

```
hetumind-nodes/
├── cluster/ai_agent/
│   ├── graph_flow_agent.rs          # AI Agent 工作流管理
│   ├── ai_agent_v1.rs             # 原始实现（保留）
│   └── parameters.rs              # 共享参数定义
├── llm/deepseek_node/
│   ├── graph_flow_deepseek.rs      # DeepSeek LLM 工作流
│   └── deepseek_v1.rs            # 原始实现（保留）
└── memory/
    ├── graph_flow_memory.rs        # 内存管理任务
    └── simple_memory_node/        # 原始实现（保留）
```

## 使用指南

### 1. 内存管理

#### 创建内存管理器

```rust
use hetumind_nodes::memory::graph_flow_memory::GraphFlowMemoryManager;
use std::sync::Arc;

let memory_manager = Arc::new(GraphFlowMemoryManager::new());
```

#### 存储消息

```rust
use serde_json::json;

let messages = vec![
    json!({
        "role": "user",
        "content": "Hello, how are you?",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }),
    json!({
        "role": "assistant",
        "content": "I'm doing well, thank you!",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }),
];

let memory_data = memory_manager
    .store_messages(
        "session_123",
        "workflow_456",
        messages,
        None // 使用默认配置
    )
    .await?;
```

#### 检索消息

```rust
let recent_messages = memory_manager
    .retrieve_messages("session_123", 10) // 检索最近10条消息
    .await?;

for msg in recent_messages {
    println!("[{}] {}", msg.role, msg.content);
}
```

### 2. DeepSeek LLM 调用

#### 创建 LLM 管理器

```rust
use hetumind_nodes::llm::deepseek_node::graph_flow_deepseek::GraphFlowDeepSeekManager;

let deepseek_manager = GraphFlowDeepSeekManager::new();
```

#### 执行 LLM 调用

```rust
use hetumind_nodes::llm::deepseek_node::graph_flow_deepseek::GraphFlowDeepSeekConfig;

let config = GraphFlowDeepSeekConfig {
    model: "deepseek-chat".to_string(),
    max_tokens: Some(1000),
    temperature: Some(0.7),
    ..Default::default()
};

let input = json!({
    "prompt": "What are the benefits of using Rust?",
});

let response = deepseek_manager
    .execute_llm_call(config, input, api_key)
    .await?;

println!("LLM Response: {}", response);
```

### 3. AI Agent 工作流

#### 创建 AI Agent 管理器

```rust
use hetumind_nodes::cluster::ai_agent::graph_flow_agent::GraphFlowAgentManager;

let agent_manager = GraphFlowAgentManager::new();
```

#### 执行完整工作流

```rust
use hetumind_nodes::cluster::ai_agent::graph_flow_agent::GraphFlowAgentConfig;
use hetumind_nodes::cluster::ai_agent::parameters::AiAgentConfig;

let agent_config = GraphFlowAgentConfig {
    base_config: AiAgentConfig {
        system_prompt: Some("You are a helpful Rust programming assistant.".to_string()),
        max_iterations: Some(3),
        temperature: Some(0.7),
        enable_streaming: Some(false),
        memory_config: Some(hetumind_nodes::cluster::ai_agent::parameters::MemoryConfig {
            max_history: Some(10),
            context_window: Some(5),
            persistence_enabled: Some(false),
        }),
    },
    session_id: "session_123".to_string(),
    memory_config: Some(GraphFlowMemoryConfig::default()),
    llm_config: Some(json!({
        "model": "deepseek-chat",
        "temperature": 0.7,
    })),
    tools_config: None,
};

let user_input = json!({
    "content": "Explain Rust's ownership system.",
});

let response = agent_manager
    .execute_agent_workflow(
        agent_config,
        user_input,
        api_key,
        "deepseek-chat".to_string(),
    )
    .await?;

println!("Agent Response: {}", response);
```

## 工作流详解

### AI Agent 工作流

AI Agent 工作流包含以下任务：

1. **GraphFlowMessagePreparationTask**: 准备消息，包括系统提示词和历史对话
2. **GraphFlowMemoryRetrieveTask**: 从内存中检索历史消息
3. **GraphFlowLLMTask**: 调用 LLM 获取响应
4. **GraphFlowMemoryStoreTask**: 将对话保存到内存
5. **GraphFlowResponsePostProcessTask**: 处理最终响应

### DeepSeek LLM 工作流

DeepSeek LLM 工作流包含以下任务：

1. **GraphFlowDeepSeekPreprocessTask**: 验证和预处理输入
2. **GraphFlowDeepSeekLLMTask**: 执行实际的 LLM 调用
3. **GraphFlowDeepSeekPostProcessTask**: 格式化和后处理响应

### 内存管理工作流

内存管理工作流包含以下任务：

1. **GraphFlowMemoryStoreTask**: 存储消息到内存
2. **GraphFlowMemoryRetrieveTask**: 从内存检索消息

## 配置选项

### 内存配置

```rust
use hetumind_nodes::memory::graph_flow_memory::GraphFlowMemoryConfig;

let memory_config = GraphFlowMemoryConfig {
    session_id: "my_session".to_string(),
    context_window_length: 10,        // 保存最近10条消息
    persistence_enabled: false,        // 不持久化到磁盘
    input_key: "input".to_string(),
    memory_key: "chat_history".to_string(),
    output_key: "output".to_string(),
};
```

### LLM 配置

```rust
use hetumind_nodes::llm::deepseek_node::graph_flow_deepseek::GraphFlowDeepSeekConfig;
use hetumind_nodes::llm::shared::CommonLlmParameters;

let llm_config = GraphFlowDeepSeekConfig {
    model: "deepseek-chat".to_string(),
    max_tokens: Some(2000),
    temperature: Some(0.8),
    top_p: Some(90),
    stop_sequences: Some(vec!["</stop>".to_string()]),
    common: CommonLlmParameters {
        api_key: "your_api_key".to_string(),
        // ... 其他通用参数
    },
    workflow_id: "my_workflow".to_string(),
    session_id: "my_session".to_string(),
};
```

### AI Agent 配置

```rust
use hetumind_nodes::cluster::ai_agent::graph_flow_agent::GraphFlowAgentConfig;
use hetumind_nodes::cluster::ai_agent::parameters::AiAgentConfig;

let agent_config = GraphFlowAgentConfig {
    base_config: AiAgentConfig {
        system_prompt: Some("Custom system prompt".to_string()),
        max_iterations: Some(5),
        temperature: Some(0.6),
        enable_streaming: Some(false),
        memory_config: Some(hetumind_nodes::cluster::ai_agent::parameters::MemoryConfig {
            max_history: Some(20),
            context_window: Some(10),
            persistence_enabled: Some(true),
        }),
    },
    session_id: "my_session".to_string(),
    memory_config: Some(GraphFlowMemoryConfig::default()),
    llm_config: Some(json!({
        "model": "deepseek-chat",
        "temperature": 0.6,
    })),
    tools_config: Some(vec![/* 工具配置 */]),
};
```

## 错误处理

所有管理器方法都返回 `Result<T, Box<dyn std::error::Error + Send + Sync>>`，建议进行适当的错误处理：

```rust
match memory_manager.retrieve_messages("session_123", 5).await {
    Ok(messages) => {
        for msg in messages {
            println!("{}", msg.content);
        }
    }
    Err(e) => {
        eprintln!("Failed to retrieve messages: {}", e);
        // 处理错误情况
    }
}
```

## 性能考虑

### 内存管理

- 使用滑动窗口算法管理内存，自动清理旧消息
- 会话隔离确保不同会话的内存不会相互影响
- 内存数据存储在内存中，适合短期使用

### LLM 调用

- 支持异步并发调用
- 自动重试机制（可配置）
- 响应缓存（可选）

### 工作流执行

- 任务并行执行（当可能时）
- 状态管理和恢复
- 资源自动清理

## 测试

### 运行示例

```bash
# 设置环境变量
export DEEPSEEK_API_KEY="your_api_key_here"

# 运行集成示例
cargo run --example graph_flow_integration_example
```

### 运行基准测试

```bash
# 运行性能基准测试
cargo bench --bench graph_flow_benchmark
```

### 运行单元测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test --package hetumind-nodes --lib graph_flow
```

## 扩展指南

### 添加新的 LLM 提供者

1. 创建新的 `GraphFlowLLMProvider` 实现
2. 实现 `Task` trait
3. 创建对应的管理器
4. 添加到工作流图中

### 添加新的工具

1. 定义工具配置结构
2. 创建工具执行任务
3. 集成到 AI Agent 工作流中

### 添加新的内存后端

1. 实现 `MemoryBackend` trait
2. 创建相应的存储和检索任务
3. 更新内存管理器

## 最佳实践

1. **会话管理**: 为每个用户或对话创建独立的会话ID
2. **错误处理**: 始终处理可能的错误情况
3. **资源清理**: 及时清理不再需要的会话数据
4. **配置验证**: 在使用前验证配置参数
5. **日志记录**: 使用适当的日志级别记录重要事件

## 故障排除

### 常见问题

1. **API 密钥错误**: 确保设置了正确的环境变量
2. **内存不足**: 调整 `context_window_length` 参数
3. **工作流超时**: 增加 `max_iterations` 或检查网络连接
4. **序列化错误**: 确保 JSON 格式正确

### 调试技巧

1. 启用详细日志记录：`RUST_LOG=debug`
2. 检查会话状态：使用 `get_memory_stats()` 方法
3. 验证配置：打印配置对象进行检查
4. 单步调试：单独测试每个组件

## 迁移指南

### 从原始实现迁移

1. **保留原始实现**: 原始节点仍然可用，可以逐步迁移
2. **渐进式迁移**: 先在非关键路径上测试新实现
3. **性能对比**: 使用基准测试比较性能
4. **功能验证**: 确保所有功能都正常工作

### API 兼容性

新实现的 API 与原始实现保持兼容，主要区别：

1. 使用管理器模式替代直接节点调用
2. 配置对象结构略有不同
3. 错误类型更统一
4. 支持更多配置选项

## 未来计划

1. **持久化后端**: 支持数据库和文件存储
2. **分布式支持**: 支持跨节点的会话管理
3. **流式处理**: 完整的流式响应支持
4. **工具集成**: 更丰富的工具生态系统
5. **监控指标**: 内置性能监控和指标收集