# AI Agent Provider 使用指南

本文档展示了如何在Cluster Node架构中使用新实现的AI Agent SubNodeProvider。

## 概述

AI Agent Provider是Cluster Node架构中Agent SubNodeProvider的具体实现，提供了智能任务编排功能，能够协调LLM、Memory和Tool的交互，实现复杂的AI Agent行为。

## 主要特性

- ✅ **完整的SubNodeProvider接口实现**：支持Agent SubNodeProvider的所有方法
- ✅ **智能任务编排**：协调LLM、Memory和Tool的交互
- ✅ **会话管理**：支持持久化会话和历史记录
- ✅ **迭代执行**：支持多轮对话和工具调用
- ✅ **配置灵活**：支持自定义系统提示词、温度参数等
- ✅ **使用统计**：提供详细的执行统计信息
- ✅ **错误处理**：完善的错误处理和恢复机制
- ✅ **可扩展架构**：支持添加新的LLM和Memory Provider

## 基本使用

### 1. 创建AI Agent Provider

```rust
use hetumind_core::workflow::providers::{AiAgentProvider, AiAgentProviderConfig, create_ai_agent_provider};

// 使用默认配置
let provider = AiAgentProvider::new(AiAgentProviderConfig::default());

// 使用自定义配置
let config = AiAgentProviderConfig {
    default_system_prompt: "你是一个专业的AI助手，专门帮助用户解决问题。".to_string(),
    max_iterations: 15,
    default_temperature: 0.8,
    enable_streaming: false,
    enable_tools: true,
    session_timeout_seconds: 7200, // 2 hours
};

let provider = AiAgentProvider::new(config);

// 或者使用工厂函数
let provider = create_ai_agent_provider(Some(config))?;
```

### 2. 初始化Provider

```rust
// 初始化Provider
provider.initialize().await?;
```

### 3. 执行Agent任务

```rust
use hetumind_core::workflow::sub_node_provider::{AgentConfig, Message};

// 准备输入消息
let messages = vec![
    Message {
        role: "user".to_string(),
        content: "你好，请帮我分析一下这个项目的架构。".to_string(),
    }
];

// 配置Agent执行
let agent_config = AgentConfig {
    system_prompt: Some("你是一个软件架构师助手。".to_string()),
    max_iterations: Some(5),
    temperature: Some(0.7),
    enable_streaming: Some(false),
    enable_tools: Some(true),
    session_id: Some("architecture_analysis_session".to_string()),
};

// 执行Agent任务
let response = provider.execute_agent(messages, agent_config).await?;

println!("Agent回复: {}", response.content);
println!("执行统计: {:?}", response.usage);
println!("会话信息: {:?}", response.session_info);
```

## 高级用法

### 1. 与LLM Provider集成

```rust
use hetumind_core::workflow::providers::{DeepSeekLLMProvider, DeepSeekConfig};

// 创建LLM Provider
let llm_config = DeepSeekConfig {
    model: "deepseek-chat".to_string(),
    api_key: Some("your-api-key".to_string()),
    max_tokens: Some(4000),
    temperature: Some(0.7),
    ..Default::default()
};
let llm_provider = Arc::new(DeepSeekLLMProvider::new(llm_config));

// 创建带有LLM的AI Agent
let agent_provider = AiAgentProvider::new(AiAgentProviderConfig::default())
    .with_llm_provider(llm_provider);

agent_provider.initialize().await?;
```

### 2. 与Memory Provider集成

```rust
use hetumind_core::workflow::providers::{MemoryProvider, MemoryProviderConfig};

// 创建Memory Provider
let memory_config = MemoryProviderConfig {
    max_messages: 100,
    persistence_enabled: true,
    session_timeout_seconds: 3600,
    cleanup_interval_seconds: 300,
};
let memory_provider = Arc::new(MemoryProvider::new(memory_config));

// 创建带有Memory的AI Agent
let agent_provider = AiAgentProvider::new(AiAgentProviderConfig::default())
    .with_memory_provider(memory_provider);

agent_provider.initialize().await?;
```

### 3. 与NodeRegistry集成

```rust
use hetumind_core::workflow::{NodeRegistry, NodeKind};

// 创建NodeRegistry
let node_registry = NodeRegistry::new();

// 注册AI Agent Provider
let node_kind: NodeKind = "ai_agent_provider".into();
node_registry.register_subnode_provider(node_kind.clone(), provider)?;

// 验证注册成功
assert!(node_registry.has_subnode_provider(&node_kind));
assert_eq!(node_registry.subnode_provider_count(), 1);

// 获取注册的Provider
let retrieved_provider = node_registry.get_subnode_provider(&node_kind)?;
```

### 4. 与GraphFlow任务集成

```rust
use hetumind_core::workflow::{
    graph_flow_tasks::{ClusterNodeExecutor, Context},
    sub_node_provider::{ClusterNodeConfig, ExecutionConfig, AgentConfig},
};

// 创建ClusterNodeExecutor
let mut executor = ClusterNodeExecutor::new(node_registry);

// 配置ClusterNode
let cluster_config = ClusterNodeConfig {
    agent_config: Some(AgentConfig {
        system_prompt: Some("你是一个专业的数据分析助手。".to_string()),
        max_iterations: Some(10),
        temperature: Some(0.5),
        enable_streaming: Some(false),
        enable_tools: Some(true),
        session_id: Some("data_analysis_session".to_string()),
    }),
    llm_config: None,
    memory_config: None,
    tools_config: None,
    execution_config: ExecutionConfig {
        timeout_seconds: Some(60),
        max_retries: Some(3),
        parallel_execution: Some(false),
    },
};

// 注册Provider到Executor
executor.register_subnode_provider(node_kind, cluster_config)?;

// 执行任务
let task_ids = executor.task_ids();
let mut context = Context::new();
context.set("input_messages", &serde_json::json!([
    {
        "role": "user",
        "content": "请分析这组数据的趋势：[1, 3, 5, 7, 9, 11]"
    }
]))?;

let result = executor.execute_task(&task_ids[0], context).await?;
println!("任务结果: {:?}", result.response);
```

### 5. 配置管理

```rust
// 动态更新配置
let new_config = AiAgentProviderConfig {
    default_system_prompt: "你是一个专业的AI顾问。".to_string(),
    max_iterations: 20,
    default_temperature: 0.6,
    enable_streaming: true,
    enable_tools: false,
    session_timeout_seconds: 14400, // 4 hours
};

provider.update_config(new_config);

// 验证配置更新
assert_eq!(provider.config().max_iterations, 20);
assert!(provider.config().enable_streaming);
```

## 配置选项

### AiAgentProviderConfig

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `default_system_prompt` | `String` | `"You are a helpful AI assistant with access to various tools and memory."` | 默认系统提示词 |
| `max_iterations` | `u32` | `10` | 最大迭代次数 |
| `default_temperature` | `f64` | `0.7` | 默认温度参数 |
| `enable_streaming` | `bool` | `false` | 是否启用流式响应 |
| `enable_tools` | `bool` | `true` | 是否启用工具调用 |
| `session_timeout_seconds` | `u64` | `3600` | 会话超时时间（秒） |

### AgentConfig

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `system_prompt` | `Option<String>` | `None` | 系统提示词 |
| `max_iterations` | `Option<u32>` | `None` | 最大迭代次数 |
| `temperature` | `Option<f64>` | `None` | 温度参数 |
| `enable_streaming` | `Option<bool>` | `None` | 是否启用流式响应 |
| `enable_tools` | `Option<bool>` | `None` | 是否启用工具调用 |
| `session_id` | `Option<String>` | `None` | 会话ID |

## 执行状态和统计

### AgentUsageStats

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageStats {
    pub total_iterations: u32,        // 总迭代次数
    pub llm_calls: u32,              // LLM调用次数
    pub tool_calls: u32,             // 工具调用次数
    pub total_tokens: Option<u32>,   // 总令牌数（如果可用）
}
```

### SessionInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,      // 会话ID
    pub history_length: usize,   // 历史消息数
    pub has_memory: bool,        // 是否使用内存
}
```

## 工作流程

### AI Agent执行流程

1. **初始化**：创建执行状态，解析会话ID
2. **系统提示词准备**：根据配置准备系统提示词
3. **历史检索**：从Memory Provider检索历史对话
4. **消息组装**：组合系统提示词、历史消息和当前输入
5. **迭代执行**：
   - 调用LLM生成响应
   - 检查是否需要工具调用
   - 执行工具调用（如果有）
   - 将结果添加到对话历史
   - 检查是否达到终止条件
6. **结果存储**：将对话保存到Memory Provider
7. **返回响应**：生成包含统计信息的响应

### 执行状态管理

```rust
#[derive(Debug, Clone)]
struct AgentExecutionState {
    current_iteration: u32,
    total_llm_calls: u32,
    total_tool_calls: u32,
    start_time: DateTime<Utc>,
    session_id: Option<String>,
}
```

## 错误处理

所有操作都返回`Result<T, NodeExecutionError>`，常见的错误类型包括：

```rust
// 配置错误
NodeExecutionError::ConfigurationError("Invalid agent configuration")

// 执行错误
NodeExecutionError::ExecutionError("Agent execution failed")

// 数据处理错误
NodeExecutionError::DataProcessingError { message: "Failed to process agent response" }

// 超时错误
NodeExecutionError::Timeout("Agent execution timeout")
```

## 工厂函数

### create_ai_agent_provider

```rust
// 使用默认配置创建
let provider = create_ai_agent_provider(None)?;

// 使用自定义配置创建
let config = AiAgentProviderConfig {
    max_iterations: 15,
    enable_tools: true,
    ..Default::default()
};
let provider = create_ai_agent_provider(Some(config))?;
```

### create_ai_agent_provider_from_config

```rust
// 从AgentConfig创建
let agent_config = AgentConfig {
    system_prompt: Some("You are a helpful assistant.".to_string()),
    max_iterations: Some(20),
    temperature: Some(0.8),
    enable_tools: Some(true),
    ..Default::default()
};

let provider = create_ai_agent_provider_from_config(agent_config)?;
```

## 节点定义

AI Agent Provider会自动创建相应的NodeDefinition：

- **节点类型**: `ai_agent_provider`
- **节点组**: `Transform` (数据转换)
- **版本**: `1.0.0`
- **图标**: `robot`
- **图标颜色**: 紫色
- **文档**: https://docs.hetumind.ai/ai-agent

## 测试

运行测试：

```bash
cargo test -p hetumind-core ai_agent_provider
```

测试包括：
- 配置转换测试
- Provider创建测试
- 初始化测试
- Agent执行测试
- 执行状态管理测试
- 集成测试

## 示例代码

### 完整示例

```rust
use hetumind_core::workflow::{
    providers::{AiAgentProvider, AiAgentProviderConfig, DeepSeekLLMProvider, MemoryProvider},
    sub_node_provider::{AgentConfig, Message, DeepSeekConfig, MemoryProviderConfig},
    NodeRegistry,
    NodeKind,
    graph_flow_tasks::{ClusterNodeExecutor, Context, ClusterNodeConfig},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建LLM Provider
    let llm_config = DeepSeekConfig {
        model: "deepseek-chat".to_string(),
        api_key: Some("your-api-key".to_string()),
        max_tokens: Some(2000),
        temperature: Some(0.7),
        ..Default::default()
    };
    let llm_provider = Arc::new(DeepSeekLLMProvider::new(llm_config));

    // 2. 创建Memory Provider
    let memory_config = MemoryProviderConfig {
        max_messages: 50,
        persistence_enabled: true,
        session_timeout_seconds: 1800, // 30 minutes
        ..Default::default()
    };
    let memory_provider = Arc::new(MemoryProvider::new(memory_config));

    // 3. 创建AI Agent Provider
    let agent_config = AiAgentProviderConfig {
        default_system_prompt: "你是一个专业的AI助手，能够帮助用户解决各种问题。".to_string(),
        max_iterations: 8,
        default_temperature: 0.7,
        enable_tools: true,
        enable_streaming: false,
        session_timeout_seconds: 3600,
    };

    let agent_provider = AiAgentProvider::new(agent_config)
        .with_llm_provider(llm_provider)
        .with_memory_provider(memory_provider);

    // 4. 初始化Provider
    agent_provider.initialize().await?;

    // 5. 注册到NodeRegistry
    let node_registry = NodeRegistry::new();
    let node_kind = "ai_agent_provider".into();
    node_registry.register_subnode_provider(node_kind.clone(), Arc::new(agent_provider))?;

    // 6. 创建ClusterNodeExecutor
    let mut executor = ClusterNodeExecutor::new(node_registry);

    // 7. 配置ClusterNode
    let cluster_config = ClusterNodeConfig {
        agent_config: Some(AgentConfig {
            system_prompt: Some("你是一个数据分析专家。".to_string()),
            max_iterations: Some(5),
            temperature: Some(0.6),
            enable_tools: Some(true),
            session_id: Some("data_analysis_session".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    // 8. 注册到Executor
    executor.register_subnode_provider(node_kind, cluster_config)?;

    // 9. 准备输入消息
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "请帮我分析以下数据的趋势：[10, 15, 12, 18, 25, 30, 28]".to_string(),
        }
    ];

    // 10. 执行任务
    let task_ids = executor.task_ids();
    let mut context = Context::new();
    context.set("input_messages", &messages)?;

    let result = executor.execute_task(&task_ids[0], context).await?;

    println!("✅ AI Agent Provider集成测试成功！");
    println!("📊 分析结果: {:?}", result.response);

    Ok(())
}
```

### 简单使用示例

```rust
use hetumind_core::workflow::{
    providers::{AiAgentProvider, AiAgentProviderConfig},
    sub_node_provider::{AgentConfig, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建Provider
    let provider = AiAgentProvider::new(AiAgentProviderConfig::default());
    provider.initialize().await?;

    // 准备消息
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "你好，请介绍一下你自己。".to_string(),
        }
    ];

    // 配置Agent
    let config = AgentConfig {
        system_prompt: Some("你是一个友好的AI助手。".to_string()),
        max_iterations: Some(3),
        ..Default::default()
    };

    // 执行任务
    let response = provider.execute_agent(messages, config).await?;

    println!("Agent回复: {}", response.content);

    if let Some(usage) = response.usage {
        println!("执行统计: 迭代{}次，LLM调用{}次", usage.total_iterations, usage.llm_calls);
    }

    Ok(())
}
```

## 最佳实践

1. **合理设置迭代次数**：根据任务复杂度设置合适的`max_iterations`
2. **优化系统提示词**：设计清晰、具体的系统提示词
3. **会话管理**：合理设置会话超时时间，避免内存泄漏
4. **错误处理**：始终检查并妥善处理可能的错误
5. **监控统计信息**：关注执行统计，优化性能
6. **工具集成**：根据需要启用或禁用工具调用功能
7. **温度调节**：根据任务类型调整温度参数

## 性能考虑

1. **迭代控制**：避免无限循环，设置合理的最大迭代次数
2. **内存管理**：定期清理过期会话，控制内存使用
3. **并发执行**：支持多个Agent实例并发运行
4. **缓存优化**：缓存常用的LLM响应和工具结果
5. **超时设置**：设置合适的超时时间，避免长时间等待

## 扩展功能

未来可能添加的功能：

- **多模态支持**：支持图像、音频等多模态输入
- **工具插件系统**：支持动态加载和卸载工具
- **分布式执行**：支持跨多个节点的分布式Agent执行
- **流式响应**：实现真正的流式响应功能
- **Agent协作**：支持多个Agent之间的协作

通过遵循本指南，您可以成功地在Cluster Node架构中集成和使用AI Agent Provider。