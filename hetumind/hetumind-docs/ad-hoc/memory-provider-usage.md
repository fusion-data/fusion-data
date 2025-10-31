# Memory Provider 使用指南

本文档展示了如何在Cluster Node架构中使用新实现的Memory SubNodeProvider。

## 概述

Memory Provider是Cluster Node架构中Memory SubNodeProvider的具体实现，提供了会话消息存储和检索功能，支持多会话管理和滑动窗口内存管理。

## 主要特性

- ✅ **完整的SubNodeProvider接口实现**：支持Memory SubNodeProvider的所有方法
- ✅ **会话管理**：支持多会话隔离和管理
- ✅ **消息存储和检索**：支持消息的历史记录和检索
- ✅ **滑动窗口内存管理**：自动管理内存限制，防止内存溢出
- ✅ **会话超时清理**：自动清理过期会话，防止内存泄漏
- ✅ **统计信息**：提供会话统计和使用情况信息
- ✅ **线程安全**：使用tokio::sync::Mutex确保并发安全
- ✅ **配置管理**：支持自定义配置参数

## 基本使用

### 1. 创建Memory Provider

```rust
use hetumind_core::workflow::providers::{MemoryProvider, MemoryProviderConfig, create_memory_provider};

// 使用默认配置
let provider = MemoryProvider::new(MemoryProviderConfig::default());

// 使用自定义配置
let config = MemoryProviderConfig {
    max_messages: 200,
    persistence_enabled: true,
    session_timeout_seconds: 7200, // 2 hours
    cleanup_interval_seconds: 600, // 10 minutes
};

let provider = MemoryProvider::new(config);

// 或者使用工厂函数
let provider = create_memory_provider(Some(config))?;
```

### 2. 初始化Provider

```rust
// 初始化Provider并启动后台清理任务
provider.initialize().await?;
```

### 3. 存储和检索消息

```rust
use hetumind_core::workflow::sub_node_provider::Message;

// 准备消息
let messages = vec![
    Message {
        role: "user".to_string(),
        content: "你好，请介绍一下Rust编程语言。".to_string(),
    },
    Message {
        role: "assistant".to_string(),
        content: "Rust是一种系统编程语言，注重安全、并发和性能。".to_string(),
    },
];

// 存储消息到会话
let session_id = "user_session_123";
provider.store_messages(session_id, messages).await?;

// 从会话检索消息
let retrieved_messages = provider.retrieve_messages(session_id, 10).await?;
println!("检索到 {} 条消息", retrieved_messages.len());
```

## 高级用法

### 1. 会话管理

```rust
// 获取会话统计信息
let stats = provider.get_session_stats("session_id").await;
if let Some(stats) = stats {
    println!("会话ID: {}", stats.session_id);
    println!("消息数量: {}", stats.message_count);
    println!("最后访问时间: {:?}", stats.last_accessed);
}

// 获取所有活跃会话统计
let all_stats = provider.get_all_session_stats().await;
println!("当前有 {} 个活跃会话", all_stats.len());

// 清理特定会话
provider.clear_session("session_id").await?;

// 清理所有会话
provider.clear_all_sessions().await?;
```

### 2. 与NodeRegistry集成

```rust
use hetumind_core::workflow::{NodeRegistry, NodeKind};

// 创建NodeRegistry
let node_registry = NodeRegistry::new();

// 注册Memory Provider
let node_kind: NodeKind = "memory_provider".into();
node_registry.register_subnode_provider(node_kind.clone(), provider)?;

// 验证注册成功
assert!(node_registry.has_subnode_provider(&node_kind));
assert_eq!(node_registry.subnode_provider_count(), 1);

// 获取注册的Provider
let retrieved_provider = node_registry.get_subnode_provider(&node_kind)?;
```

### 3. 与GraphFlow任务集成

```rust
use hetumind_core::workflow::{
    graph_flow_tasks::{ClusterNodeExecutor, Context},
    sub_node_provider::{ClusterNodeConfig, ExecutionConfig, MemoryConfig},
};

// 创建ClusterNodeExecutor
let mut executor = ClusterNodeExecutor::new(node_registry);

// 配置ClusterNode
let cluster_config = ClusterNodeConfig {
    memory_config: Some(MemoryConfig {
        context_window: Some(50),
        max_history: Some(100),
        persistence_enabled: Some(true),
    }),
    llm_config: None,
    tools_config: None,
    execution_config: ExecutionConfig {
        timeout_seconds: Some(30),
        max_retries: Some(3),
        parallel_execution: Some(true),
    },
};

// 注册Provider到Executor
executor.register_subnode_provider(node_kind, cluster_config)?;

// 执行任务
let task_ids = executor.task_ids();
let mut context = Context::new();
context.set("session_id", "test_session")?;

let result = executor.execute_task(&task_ids[0], context).await?;
println!("任务结果: {:?}", result.response);
```

### 4. 配置管理

```rust
// 动态更新配置
let new_config = MemoryProviderConfig {
    max_messages: 500,
    session_timeout_seconds: 14400, // 4 hours
    persistence_enabled: true,
    cleanup_interval_seconds: 300,   // 5 minutes
};

provider.update_config(new_config);

// 验证配置更新
assert_eq!(provider.config().max_messages, 500);
```

## 配置选项

### MemoryProviderConfig

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `max_messages` | `usize` | `100` | 每个会话最大消息数 |
| `persistence_enabled` | `bool` | `false` | 是否启用持久化（预留功能） |
| `session_timeout_seconds` | `u64` | `3600` | 会话超时时间（秒） |
| `cleanup_interval_seconds` | `u64` | `300` | 清理任务间隔时间（秒） |

### MemoryConfig

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `context_window` | `Option<usize>` | `None` | 上下文窗口大小 |
| `max_history` | `Option<usize>` | `None` | 最大历史记录数 |
| `persistence_enabled` | `Option<bool>` | `None` | 是否启用持久化 |

## 内存管理

### 滑动窗口机制

Memory Provider使用滑动窗口机制管理内存：

```rust
// 配置每个会话最多存储100条消息
let config = MemoryProviderConfig {
    max_messages: 100,
    ..Default::default()
};

// 当存储第101条消息时，会自动删除最旧的消息
// 确保内存使用量保持在可控范围内
```

### 会话超时清理

```rust
// 配置会话超时时间为1小时
let config = MemoryProviderConfig {
    session_timeout_seconds: 3600,
    cleanup_interval_seconds: 300, // 每5分钟检查一次
    ..Default::default()
};

// Provider会自动在后台运行清理任务
// 删除超过1小时未访问的会话
```

## 消息结构

### MemoryMessage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMessage {
    pub role: String,        // 消息角色（user, assistant, system等）
    pub content: String,      // 消息内容
    pub timestamp: DateTime<Utc>, // 时间戳
    pub session_id: String,   // 会话ID
}
```

### SessionStats

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub message_count: usize,                    // 消息数量
    pub last_accessed: Option<DateTime<Utc>>,    // 最后访问时间
    pub session_id: String,                      // 会话ID
}
```

## 错误处理

所有操作都返回`Result<T, NodeExecutionError>`，常见的错误类型包括：

```rust
// 会话相关错误
NodeExecutionError::InvalidInput("Session ID cannot be empty")

// 数据处理错误
NodeExecutionError::DataProcessingError { message: "Failed to store message" }

// 配置错误
NodeExecutionError::ConfigurationError("Invalid memory configuration")
```

## 工厂函数

### create_memory_provider

```rust
// 使用默认配置创建
let provider = create_memory_provider(None)?;

// 使用自定义配置创建
let config = MemoryProviderConfig {
    max_messages: 200,
    ..Default::default()
};
let provider = create_memory_provider(Some(config))?;
```

### create_memory_provider_from_config

```rust
// 从MemoryConfig创建
let memory_config = MemoryConfig {
    max_history: Some(150),
    persistence_enabled: Some(true),
    ..Default::default()
};

let provider = create_memory_provider_from_config(memory_config)?;
```

## 节点定义

Memory Provider会自动创建相应的NodeDefinition：

- **节点类型**: `memory_provider`
- **节点组**: `Transform` (数据转换)
- **版本**: `1.0.0`
- **图标**: `database`
- **图标颜色**: 绿色
- **文档**: https://docs.hetumind.ai/memory

## 测试

运行测试：

```bash
cargo test -p hetumind-core memory_provider
```

测试包括：
- 配置转换测试
- Provider创建测试
- 初始化测试
- 消息存储和检索测试
- 会话管理测试
- 集成测试

## 示例代码

### 完整示例

```rust
use hetumind_core::workflow::{
    providers::{MemoryProvider, MemoryProviderConfig},
    sub_node_provider::{MemoryConfig, Message},
    NodeRegistry,
    NodeKind,
    graph_flow_tasks::{ClusterNodeExecutor, Context, ClusterNodeConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建配置
    let config = MemoryProviderConfig {
        max_messages: 200,
        session_timeout_seconds: 7200, // 2 hours
        persistence_enabled: true,
        cleanup_interval_seconds: 600,  // 10 minutes
    };

    // 2. 创建Provider
    let provider = MemoryProvider::new(config);

    // 3. 初始化
    provider.initialize().await?;

    // 4. 注册到NodeRegistry
    let node_registry = NodeRegistry::new();
    let node_kind = "memory_provider".into();
    node_registry.register_subnode_provider(node_kind.clone(), provider.clone())?;

    // 5. 创建ClusterNodeExecutor
    let mut executor = ClusterNodeExecutor::new(node_registry);

    // 6. 配置ClusterNode
    let cluster_config = ClusterNodeConfig {
        memory_config: Some(MemoryConfig {
            context_window: Some(50),
            max_history: Some(100),
            persistence_enabled: Some(true),
        }),
        ..Default::default()
    };

    // 7. 注册到Executor
    executor.register_subnode_provider(node_kind, cluster_config)?;

    // 8. 测试消息存储和检索
    let session_id = "demo_session";
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "你好，我是新用户".to_string(),
        },
        Message {
            role: "assistant".to_string(),
            content: "你好！很高兴为您服务".to_string(),
        },
    ];

    // 存储消息
    provider.store_messages(session_id, messages.clone()).await?;

    // 检索消息
    let retrieved = provider.retrieve_messages(session_id, 10).await?;
    println!("✅ 存储了 {} 条消息", messages.len());
    println!("✅ 检索到 {} 条消息", retrieved.len());

    // 9. 获取会话统计
    let stats = provider.get_session_stats(session_id).await;
    if let Some(stats) = stats {
        println!("✅ 会话统计: {} 条消息", stats.message_count);
    }

    // 10. 执行GraphFlow任务
    let task_ids = executor.task_ids();
    let mut context = Context::new();
    context.set("session_id", session_id)?;

    let result = executor.execute_task(&task_ids[0], context).await?;
    println!("✅ Memory Provider集成测试成功！");
    println!("📊 结果: {:?}", result.response);

    Ok(())
}
```

## 最佳实践

1. **合理设置消息限制**：根据内存容量和性能需求设置合适的`max_messages`
2. **配置会话超时**：根据业务需求设置合理的`session_timeout_seconds`
3. **监控内存使用**：定期检查活跃会话数量和消息总数
4. **清理过期会话**：利用自动清理功能防止内存泄漏
5. **线程安全**：Provider已实现线程安全，可在多线程环境中使用
6. **错误处理**：始终检查并妥善处理可能的错误
7. **测试驱动**：为关键功能编写单元测试和集成测试

## 性能考虑

1. **内存优化**：使用滑动窗口机制限制内存使用
2. **并发性能**：使用tokio::sync::Mutex优化并发访问
3. **清理效率**：后台定期清理避免内存积累
4. **索引优化**：使用HashMap快速查找会话数据

## 扩展功能

未来可能添加的功能：

- **持久化存储**：支持数据库持久化
- **分布式缓存**：支持Redis等分布式缓存
- **消息压缩**：对旧消息进行压缩存储
- **智能清理**：基于访问频率的智能清理策略
- **多租户支持**：支持租户级别的数据隔离

通过遵循本指南，您可以成功地在Cluster Node架构中集成和使用Memory Provider。