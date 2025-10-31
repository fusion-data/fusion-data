# DeepSeek LLM Provider 使用指南

本文档展示了如何在Cluster Node架构中使用新实现的DeepSeek LLM SubNodeProvider。

## 概述

DeepSeek LLM Provider是Cluster Node架构中LLM SubNodeProvider的具体实现，提供了与DeepSeek API集成的完整功能。

## 主要特性

- ✅ **完整的SubNodeProvider接口实现**：支持LLM SubNodeProvider的所有方法
- ✅ **配置管理**：支持模型选择、API密钥、温度等参数配置
- ✅ **NodeRegistry集成**：可以注册到NodeRegistry进行统一管理
- ✅ **GraphFlow任务集成**：可以作为GraphFlow任务执行
- ✅ **类型安全**：使用Rust类型系统确保编译时安全
- ✅ **异步支持**：所有操作都是异步的
- ✅ **错误处理**：完善的错误处理机制

## 基本使用

### 1. 创建DeepSeek Provider

```rust
use hetumind_core::workflow::providers::{DeepSeekLLMProvider, DeepSeekConfig, create_deepseek_provider};

// 使用默认配置
let provider = DeepSeekLLMProvider::new(DeepSeekConfig::default());

// 使用自定义配置
let config = DeepSeekConfig {
    model: "deepseek-chat".to_string(),
    api_key: Some("your-api-key".to_string()),
    max_tokens: Some(4096),
    temperature: Some(0.7),
    top_p: Some(95.0),
    ..Default::default()
};

let provider = DeepSeekLLMProvider::new(config);

// 或者使用工厂函数
let provider = create_deepseek_provider(Some(config))?;
```

### 2. 初始化Provider

```rust
// 初始化Provider会验证API密钥的有效性
provider.initialize().await?;

// 如果API密钥无效，会返回NodeExecutionError
```

### 3. 调用LLM

```rust
use hetumind_core::workflow::sub_node_provider::{LLMConfig, Message};

// 准备消息
let messages = vec![
    Message {
        role: "user".to_string(),
        content: "你好，请介绍一下Rust编程语言。".to_string(),
    }
];

// 配置LLM调用
let llm_config = LLMConfig {
    model: "deepseek-chat".to_string(),
    max_tokens: Some(1000),
    temperature: Some(0.7),
    top_p: Some(90),
    stop_sequences: Some(vec!["\n\nHuman:".to_string()]),
    api_key: Some("your-api-key".to_string()),
};

// 调用LLM
let response = provider.call_llm(messages, llm_config).await?;

println!("回复: {}", response.content);
println!("使用统计: {:?}", response.usage);
```

## 高级用法

### 1. 与NodeRegistry集成

```rust
use hetumind_core::workflow::{NodeRegistry, NodeKind};

// 创建NodeRegistry
let node_registry = NodeRegistry::new();

// 注册DeepSeek Provider
let node_kind: NodeKind = "deepseek_llm".into();
node_registry.register_subnode_provider(node_kind.clone(), provider)?;

// 验证注册成功
assert!(node_registry.has_subnode_provider(&node_kind));
assert_eq!(node_registry.subnode_provider_count(), 1);

// 获取注册的Provider
let retrieved_provider = node_registry.get_subnode_provider(&node_kind)?;
```

### 2. 与GraphFlow任务集成

```rust
use hetumind_core::workflow::{
    graph_flow_tasks::{ClusterNodeExecutor, Context},
    sub_node_provider::{ClusterNodeConfig, ExecutionConfig},
};

// 创建ClusterNodeExecutor
let mut executor = ClusterNodeExecutor::new(node_registry);

// 配置ClusterNode
let cluster_config = ClusterNodeConfig {
    llm_config: Some(LLMConfig {
        model: "deepseek-chat".to_string(),
        max_tokens: Some(500),
        temperature: Some(0.8),
        ..Default::default()
    }),
    memory_config: None,
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
context.set("input_messages", &messages)?;

let result = executor.execute_task(&task_ids[0], context).await?;
println!("任务结果: {:?}", result.response);
```

### 3. 配置管理

```rust
// 动态更新配置
let new_config = DeepSeekConfig {
    model: "deepseek-coder".to_string(),
    max_tokens: Some(8000),
    temperature: Some(0.1),
    ..Default::default()
};

provider.update_config(new_config);

// 验证配置更新
assert_eq!(provider.config().model, "deepseek-coder");
```

## 配置选项

### DeepSeekConfig

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `model` | `String` | `"deepseek-chat"` | DeepSeek模型名称 |
| `max_tokens` | `Option<u32>` | `Some(4096)` | 最大生成token数 |
| `temperature` | `Option<f64>` | `Some(0.7)` | 温度参数(0.0-2.0) |
| `top_p` | `Option<f64>` | `Some(1.0)` | Top-p采样参数 |
| `stop_sequences` | `Option<Vec<String>>` | `None` | 停止序列 |
| `base_url` | `Option<String>` | `None` | API基础URL |
| `timeout` | `Option<u64>` | `None` | 请求超时时间(秒) |
| `api_key` | `Option<String>` | `None` | API密钥 |

### LLMConfig

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `model` | `String` | `"default"` | 模型名称 |
| `max_tokens` | `Option<u32>` | `None` | 最大token数 |
| `temperature` | `Option<f64>` | `None` | 温度参数 |
| `top_p` | `Option<u32>` | `None` | Top-p参数 |
| `stop_sequences` | `Option<Vec<String>>` | `None` | 停止序列 |
| `api_key` | `Option<String>` | `None` | API密钥 |

## 错误处理

所有操作都返回`Result<T, NodeExecutionError>`，常见的错误类型包括：

```rust
// API密钥相关错误
NodeExecutionError::ConfigurationError("DeepSeek API key not found")

// 外部服务错误
NodeExecutionError::ExternalServiceError { service: "DeepSeek API error" }

// 数据处理错误
NodeExecutionError::DataProcessingError { message: "Invalid response format" }
```

## API密钥管理

DeepSeek Provider支持多种API密钥来源：

1. **直接配置**
   ```rust
   let config = DeepSeekConfig {
       api_key: Some("your-api-key".to_string()),
       ..Default::default()
   };
   ```

2. **环境变量**
   ```bash
   export DEEPSEEK_API_KEY="your-api-key"
   ```
   Provider会自动查找`DEEPSEEK_API_KEY`环境变量。

3. **运行时动态设置**
   ```rust
   let mut provider = DeepSeekLLMProvider::new(DeepSeekConfig::default());
   // 稍后通过update_config设置API密钥
   ```

## 节点定义

DeepSeek Provider会自动创建相应的NodeDefinition：

- **节点类型**: `deepseek_llm`
- **节点组**: `Transform` (数据转换)
- **版本**: `1.0.0`
- **图标**: `robot`
- **图标颜色**: 蓝色
- **文档**: https://platform.deepseek.com/

## 测试

运行测试：

```bash
cargo test -p hetumind-core deepseek_provider
```

测试包括：
- 配置转换测试
- Provider创建测试
- 初始化测试
- 消息转换测试
- 集成测试

## 示例代码

### 完整示例

```rust
use hetumind_core::workflow::{
    providers::{DeepSeekLLMProvider, DeepSeekConfig},
    sub_node_provider::{LLMConfig, Message},
    NodeRegistry,
    NodeKind,
    graph_flow_tasks::{ClusterNodeExecutor, Context, ClusterNodeConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建配置
    let config = DeepSeekConfig {
        model: "deepseek-chat".to_string(),
        api_key: Some("your-api-key".to_string()),
        max_tokens: Some(1000),
        temperature: Some(0.7),
        ..Default::default()
    };

    // 2. 创建Provider
    let provider = DeepSeekLLMProvider::new(config);

    // 3. 初始化
    provider.initialize().await?;

    // 4. 注册到NodeRegistry
    let node_registry = NodeRegistry::new();
    let node_kind = "deepseek_llm".into();
    node_registry.register_subnode_provider(node_kind.clone(), provider.clone())?;

    // 5. 创建ClusterNodeExecutor
    let mut executor = ClusterNodeExecutor::new(node_registry);

    // 6. 配置ClusterNode
    let cluster_config = ClusterNodeConfig {
        llm_config: Some(LLMConfig {
            model: "deepseek-chat".to_string(),
            max_tokens: Some(500),
            temperature: Some(0.8),
            ..Default::default()
        }),
        ..Default::default()
    };

    // 7. 注册到Executor
    executor.register_subnode_provider(node_kind, cluster_config)?;

    // 8. 准备消息
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "请解释什么是Cluster Node架构？".to_string(),
        }
    ];

    // 9. 执行任务
    let task_ids = executor.task_ids();
    let mut context = Context::new();
    context.set("input_messages", &messages)?;

    let result = executor.execute_task(&task_ids[0], context).await?;

    println!("✅ DeepSeek LLM Provider集成测试成功！");
    println!("📊 结果: {:?}", result.response);

    Ok(())
}
```

## 最佳实践

1. **API密钥安全**：不要将API密钥硬编码在代码中，使用环境变量或安全的配置管理系统
2. **错误处理**：始终检查并妥善处理可能的错误
3. **资源管理**：及时释放不再使用的Provider实例
4. **配置验证**：在初始化时验证配置的有效性
5. **测试驱动**：为关键功能编写单元测试和集成测试

## 注意事项

- 当前版本使用模拟API调用，生产环境需要实现真实的HTTP请求
- 确保网络连接稳定，避免请求超时
- 合理设置token限制，避免超出API配额
- 定期更新API密钥，确保访问权限

## 故障排除

### 常见问题

1. **API密钥未找到**
   - 检查环境变量`DEEPSEEK_API_KEY`
   - 确认配置中的`api_key`字段

2. **初始化失败**
   - 验证API密钥格式是否正确
   - 检查网络连接

3. **配置转换错误**
   - 确认`LLMConfig`中的`top_p`是`u32`类型
   - 检查所有必需字段都已提供

4. **节点注册失败**
   - 确认节点类型唯一
   - 检查NodeRegistry状态

通过遵循本指南，您可以成功地在Cluster Node架构中集成和使用DeepSeek LLM Provider。