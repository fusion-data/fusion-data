# Cluster Node 架构集成演示

本文档展示了Cluster Node架构中所有SubNodeProvider的完整集成和协同工作。

## 架构概览

Cluster Node架构实现了以下核心组件的统一管理：

- ✅ **DeepSeek LLM Provider**: 大语言模型服务
- ✅ **Memory Provider**: 会话记忆管理
- ✅ **AI Agent Provider**: 智能任务编排
- ✅ **NodeRegistry**: 统一节点注册中心
- ✅ **ClusterNodeExecutor**: 任务执行协调器

## 集成架构图

```
┌─────────────────────────────────────────────────────────────┐
│                   Cluster Node Manager                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  DeepSeek LLM   │  │   Memory       │  │   AI Agent      │  │
│  │   Provider      │  │   Provider      │  │   Provider      │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│           │                   │                   │           │
├───────────┼───────────────────┼───────────────────┼───────────┤
│           ▼                   ▼                   ▼           │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │               NodeRegistry (统一注册)                      │  │
│  └─────────────────────────────────────────────────────────────┘  │
│           │                   │                   │           │
├───────────┼───────────────────┼───────────────────┼───────────┤
│           ▼                   ▼                   ▼           │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │             ClusterNodeExecutor (任务执行)                  │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │  │
│  │  │ LLM     │  │ Memory  │  │ Agent   │  │  Tool   │  ...   │  │
│  │  │ Task    │  │ Task    │  │ Task    │  │ Task    │       │  │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## 核心特性演示

### 1. 统一节点注册

```rust
use hetumind_core::workflow::{
    NodeRegistry, NodeKind,
    providers::{DeepSeekLLMProvider, MemoryProvider, AiAgentProvider},
};

// 创建统一注册中心
let registry = NodeRegistry::new();

// 注册所有Provider
let deepseek_kind: NodeKind = "deepseek_llm".into();
let memory_kind: NodeKind = "memory_provider".into();
let agent_kind: NodeKind = "ai_agent_provider".into();

registry.register_subnode_provider(deepseek_kind, deepseek_provider)?;
registry.register_subnode_provider(memory_kind, memory_provider)?;
registry.register_subnode_provider(agent_kind, agent_provider)?;

// 验证注册成功
assert_eq!(registry.subnode_provider_count(), 3);
```

### 2. 协同任务执行

```rust
use hetumind_core::workflow::graph_flow_tasks::ClusterNodeExecutor;

// 创建任务执行器
let executor = ClusterNodeExecutor::new(registry);

// 为每个Provider配置执行参数
executor.register_subnode_provider(
    deepseek_kind,
    ClusterNodeConfig {
        llm_config: Some(LLMConfig {
            model: "deepseek-chat".to_string(),
            temperature: Some(0.7),
            ..Default::default()
        }),
        ..Default::default()
    }
)?;

executor.register_subnode_provider(
    memory_kind,
    ClusterNodeConfig {
        memory_config: Some(MemoryConfig {
            context_window: Some(10),
            max_history: Some(100),
            ..Default::default()
        }),
        ..Default::default()
    }
)?;

executor.register_subnode_provider(
    agent_kind,
    ClusterNodeConfig {
        agent_config: Some(AgentConfig {
            system_prompt: Some("You are a helpful AI assistant.".to_string()),
            max_iterations: Some(5),
            ..Default::default()
        }),
        ..Default::default()
    }
)?;
```

### 3. 智能Agent编排

```rust
// AI Agent可以协调LLM和Memory Provider
let enhanced_agent = AiAgentProvider::new(agent_config)
    .with_llm_provider(llm_provider)    // 集成LLM能力
    .with_memory_provider(memory_provider); // 集成记忆能力

// 执行多轮对话
let session_id = "demo_session";

// 第一轮对话
let messages1 = vec![
    Message {
        role: "user".to_string(),
        content: "我的名字是张三，我正在做一个数据分析项目。".to_string(),
    }
];

let response1 = enhanced_agent.execute_agent(
    messages1,
    AgentConfig {
        session_id: Some(session_id.to_string()),
        ..Default::default()
    }
).await?;

// 第二轮对话（Agent会记住第一轮的信息）
let messages2 = vec![
    Message {
        role: "user".to_string(),
        content: "我刚才告诉你什么信息？".to_string(),
    }
];

let response2 = enhanced_agent.execute_agent(
    messages2,
    AgentConfig {
        session_id: Some(session_id.to_string()),
        ..Default::default()
    }
).await?;

// Agent会记住用户的名字和项目信息
println!("Agent回复: {}", response2.content);
// 预期输出：类似于"您告诉我您的名字是张三，正在做一个数据分析项目。"
```

### 4. 会话管理演示

```rust
// Memory Provider提供会话持久化
let memory_provider = MemoryProvider::new(MemoryProviderConfig {
    max_messages: 50,
    persistence_enabled: true,
    session_timeout_seconds: 3600,
    ..Default::default()
});

// 存储会话消息
memory_provider.store_messages(
    "user_session_123",
    vec![
        Message {
            role: "user".to_string(),
            content: "我想学习Rust编程".to_string(),
        },
        Message {
            role: "assistant".to_string(),
            content: "Rust是一门系统编程语言...".to_string(),
        }
    ]
).await?;

// 检索历史消息
let history = memory_provider.retrieve_messages("user_session_123", 10).await?;
assert_eq!(history.len(), 2);

// 获取会话统计
let stats = memory_provider.get_session_stats("user_session_123").await?;
assert!(stats.is_some());
assert_eq!(stats.unwrap().message_count, 2);
```

### 5. 性能和并发

```rust
// 支持并发执行多个Agent实例
let mut handles = Vec::new();

for i in 0..5 {
    let agent_provider = enhanced_agent.clone();
    let handle = tokio::spawn(async move {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: format!("并发测试消息 #{}", i + 1),
            }
        ];

        agent_provider.execute_agent(
            messages,
            AgentConfig {
                session_id: Some(format!("concurrent_session_{}", i)),
                ..Default::default()
            }
        ).await
    });
    handles.push(handle);
}

// 等待所有并发任务完成
for handle in handles {
    let result = handle.await.expect("Task should complete");
    assert!(result.is_ok());
}
```

## 完整工作流程演示

### 场景：智能数据分析助手

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化所有Provider
    let deepseek_provider = Arc::new(DeepSeekLLMProvider::new(DeepSeekConfig {
        model: "deepseek-chat".to_string(),
        api_key: Some("your-api-key".to_string()),
        ..Default::default()
    }));

    let memory_provider = Arc::new(MemoryProvider::new(MemoryProviderConfig {
        max_messages: 100,
        persistence_enabled: true,
        ..Default::default()
    }));

    let agent_provider = Arc::new(AiAgentProvider::new(AiAgentProviderConfig {
        default_system_prompt: "你是一个专业的数据分析助手。".to_string(),
        max_iterations: 8,
        enable_tools: true,
        ..Default::default()
    }).with_llm_provider(deepseek_provider)
      .with_memory_provider(memory_provider));

    // 2. 注册到NodeRegistry
    let registry = NodeRegistry::new();
    let agent_kind: NodeKind = "data_analysis_agent".into();
    registry.register_subnode_provider(agent_kind, agent_provider)?;

    // 3. 创建执行器
    let mut executor = ClusterNodeExecutor::new(registry);
    executor.register_subnode_provider(
        agent_kind,
        ClusterNodeConfig {
            agent_config: Some(AgentConfig {
                system_prompt: Some("你是一个专业的数据分析助手，能够帮助用户分析数据并提供洞察。".to_string()),
                max_iterations: Some(5),
                temperature: Some(0.6),
                session_id: Some("data_analysis_session".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    )?;

    // 4. 执行数据分析任务
    let task_ids = executor.task_ids();
    let mut context = executor.create_context();

    let analysis_request = json!([
        {
            "role": "user",
            "content": "请分析这组销售数据的趋势：[100, 150, 120, 180, 200, 250, 220, 300, 280, 350]"
        }
    ]);

    context.set("input_messages", analysis_request)?;

    let result = executor.execute_task(&task_ids[0], context).await?;

    println!("📊 数据分析结果: {}", result.response.unwrap_or_default());

    // 5. 继续对话（Agent会记住之前的分析）
    let mut follow_up_context = executor.create_context();

    let follow_up_request = json!([
        {
            "role": "user",
            "content": "基于刚才的分析，你建议采取什么行动？"
        }
    ]);

    follow_up_context.set("input_messages", follow_up_request)?;

    let follow_up_result = executor.execute_task(&task_ids[0], follow_up_context).await?;

    println!("💡 建议行动: {}", follow_up_result.response.unwrap_or_default());

    println!("✅ 数据分析助手演示完成！");

    Ok(())
}
```

## 技术优势

### 1. **统一架构**
- 所有Provider遵循相同的SubNodeProvider接口
- 统一的注册、配置和执行机制
- 类型安全的Provider管理

### 2. **智能编排**
- AI Agent可以协调多个Provider
- 自动化的会话管理和记忆存储
- 灵活的任务执行策略

### 3. **高性能**
- 支持并发执行
- 异步I/O操作
- 资源优化管理

### 4. **可扩展性**
- 插件化的Provider架构
- 动态注册和卸载
- 标准化的接口设计

### 5. **可靠性**
- 完善的错误处理机制
- 自动重试和恢复
- 详细的执行统计

## 实际应用场景

### 1. **智能客服系统**
```rust
// 结合LLM、Memory和Agent的智能客服
let customer_service_agent = AiAgentProvider::new(agent_config)
    .with_llm_provider(llm_provider)
    .with_memory_provider(memory_provider);

// 支持多轮对话和上下文记忆
// 自动记录客户问题和解决方案
```

### 2. **代码生成助手**
```rust
// 专业的代码生成和分析工具
let code_assistant = AiAgentProvider::new(AiAgentProviderConfig {
    default_system_prompt: "你是一个专业的编程助手...".to_string(),
    max_iterations: 10,
    enable_tools: true, // 支持代码执行工具
    ..Default::default()
});
```

### 3. **数据分析平台**
```rust
// 集成数据处理、分析和可视化
let data_analyst = AiAgentProvider::new(AiAgentProviderConfig {
    default_system_prompt: "你是一个数据科学家...".to_string(),
    ..Default::default()
});
```

## 总结

Cluster Node架构通过SubNodeProvider模式实现了：

- 🎯 **统一管理**: 所有AI能力通过统一接口管理
- 🧠 **智能编排**: AI Agent自动协调各种能力
- 💾 **持久记忆**: 完整的会话和历史管理
- ⚡ **高性能**: 并发执行和资源优化
- 🔧 **易扩展**: 插件化架构支持快速集成新能力

这个架构为构建复杂的AI应用提供了强大而灵活的基础设施。