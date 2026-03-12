# hetumind-nodes

节点执行器：Trigger、Core、LM、Store、Integration 节点。

## Imports

```rust
// Trigger 节点
use hetumind_nodes::trigger::{
    ManualTriggerNode, ScheduleTriggerNode, WebhookTriggerNode,
    EmailTriggerNode, ErrorTriggerNode, StartNode,
};

// Core 节点
use hetumind_nodes::core::{IfNode, NoOpNode, LoopOverItems};

// LM 节点
use hetumind_nodes::lm::{OpenaiModelNode, DeepSeekModelNode, MoonshotModelNode};

// Store 节点
use hetumind_nodes::store::SimpleMemoryNode;

// Integration 节点
use hetumind_nodes::integration::HttpRequestNode;
```

## Trigger Nodes

### ManualTriggerNode

```rust
pub struct ManualTriggerNodeV1 {
    definition: Arc<NodeDescription>,
}

#[async_trait]
impl FlowNode for ManualTriggerNodeV1 {
    fn description(&self) -> Arc<NodeDescription> {
        Arc::clone(&self.definition)
    }

    async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap> {
        let node = context.current_node()?;
        let config = self.parse_config(&node.parameters)?;

        if !config.enabled {
            return Err(NodeExecutionError::ParameterValidation(
                ValidationError::invalid_field_value("enabled", "手动触发器已禁用")
            ));
        }

        let trigger_data = config.generate_trigger_data();
        let data_items = ExecutionDataItems::new_items(vec![ExecutionData::new_json(trigger_data, None)]);

        Ok(make_execution_data_map(vec![
            (NodeConnectionKind::Main, vec![data_items])
        ]))
    }
}
```

### ScheduleTriggerNode

```rust
pub struct ScheduleTriggerNode {
    definition: Arc<NodeDescription>,
}

impl ScheduleTriggerNode {
    fn parse_config(&self, params: &Value) -> Result<ScheduleConfig> {
        Ok(ScheduleConfig {
            cron: params["cron"].as_str().unwrap_or("0 * * * *").to_string(),
            timezone: params["timezone"].as_str().unwrap_or("UTC").to_string(),
            enabled: params["enabled"].as_bool().unwrap_or(true),
        })
    }
}
```

### WebhookTriggerNode

```rust
pub struct WebhookTriggerNode {
    definition: Arc<NodeDescription>,
    webhook_url: String,
}

impl WebhookTriggerNode {
    pub fn generate_webhook_url(&self, workflow_id: &str, node_id: &str) -> String {
        format!("/webhook/{}/{}/trigger", workflow_id, node_id)
    }
}
```

## Core Nodes

### IfNode

```rust
pub struct IfNodeV1 {
    definition: Arc<NodeDescription>,
}

#[async_trait]
impl FlowNode for IfNodeV1 {
    async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap> {
        let node = context.current_node()?;
        let condition = node.parameters["condition"].as_str().unwrap_or("");

        // 解析并求值表达式
        let expr = parse_expression(condition)?;
        let input = context.get_input_data("main")?;
        let result = expr.evaluate(&input.first()?.json_value)?;

        let branch = if result.as_bool().unwrap_or(false) {
            NodeConnectionKind::True
        } else {
            NodeConnectionKind::False
        };

        Ok(make_execution_data_map(vec![
            (branch, input.items.clone())
        ]))
    }
}
```

### LoopOverItems

```rust
pub struct LoopOverItemsNode {
    definition: Arc<NodeDescription>,
}

#[async_trait]
impl FlowNode for LoopOverItemsNode {
    async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap> {
        let items = context.get_parameter::<Vec<Value>>("items")?;

        for item in items {
            // 对每个 item 执行子节点
            let item_data = ExecutionDataItems::new_item(ExecutionData::new_json(item, None));

            // 输出到 loop 端口
            context.output_to("loop", item_data).await?;
        }

        // 完成后输出到 done 端口
        Ok(make_execution_data_map(vec![
            (NodeConnectionKind::Done, vec![])
        ]))
    }
}
```

## LM Nodes

### OpenaiModelNode

```rust
use hetus::ai::factory::ClientFactory;

pub struct OpenaiModelNode {
    default_version: Version,
    executors: Vec<FlowNodeRef>,
}

impl Node for OpenaiModelNode {
    fn default_version(&self) -> &Version { &self.default_version }
    fn node_executors(&self) -> &[FlowNodeRef] { &self.executors }
    fn node_type(&self) -> NodeType {
        self.executors[0].description().node_type.clone()
    }
}

pub struct OpenaiModelNodeV1 {
    definition: Arc<NodeDescription>,
}

#[async_trait]
impl FlowNode for OpenaiModelNodeV1 {
    async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap> {
        let node = context.current_node()?;
        let config = self.parse_config(&node.parameters)?;

        // 获取 LLM provider
        let registry = context.get_registry();
        let provider = registry.get_llm_supplier(&NodeType::OpenAI)
            .ok_or_else(|| NodeExecutionError::ProviderNotFound("OpenAI".into()))?;

        // 获取聊天历史
        let messages = context.get_chat_messages()?;

        // 调用 LLM
        let response = provider.chat(ChatRequest {
            model: config.model,
            messages,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        }).await?;

        // 输出结果
        let output = ExecutionData::new_json(json!({
            "content": response.content,
            "usage": response.usage,
        }), None);

        Ok(make_execution_data_map(vec![
            (NodeConnectionKind::Main, vec![ExecutionDataItems::new_item(output)])
        ]))
    }
}
```

## Store Nodes

### SimpleMemoryNode

```rust
pub struct SimpleMemoryNode {
    definition: Arc<NodeDescription>,
}

#[async_trait]
impl FlowNode for SimpleMemoryNode {
    async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap> {
        let node = context.current_node()?;
        let mode = node.parameters["mode"].as_str().unwrap_or("get");

        let registry = context.get_registry();
        let memory = registry.get_memory_supplier(&NodeType::SimpleMemory)?
            .ok_or_else(|| NodeExecutionError::ProviderNotFound("Memory".into()))?;

        let session_id = context.execution_context.execution_id.to_string();

        match mode {
            "save" => {
                let messages = context.get_input_data("main")?;
                memory.store(&session_id, messages.to_chat_messages()?).await?;
            }
            "get" => {
                let limit = node.parameters["limit"].as_u64().unwrap_or(10) as usize;
                let messages = memory.retrieve(&session_id, limit).await?;

                let output = ExecutionData::new_json(json!({
                    "messages": messages,
                }), None);

                return Ok(make_execution_data_map(vec![
                    (NodeConnectionKind::Main, vec![ExecutionDataItems::new_item(output)])
                ]));
            }
            _ => {}
        }

        Ok(make_execution_data_map(vec![]))
    }
}
```

## Integration Nodes

### HttpRequestNode

```rust
pub struct HttpRequestNode {
    definition: Arc<NodeDescription>,
}

#[async_trait]
impl FlowNode for HttpRequestNode {
    async fn execute(&self, context: &NodeExecutionContext) -> Result<ExecutionDataMap> {
        let node = context.current_node()?;
        let config = self.parse_config(&node.parameters)?;

        let client = reqwest::Client::new();
        let mut request = match config.method.as_str() {
            "GET" => client.get(&config.url),
            "POST" => client.post(&config.url),
            "PUT" => client.put(&config.url),
            "DELETE" => client.delete(&config.url),
            _ => return Err(NodeExecutionError::InvalidParameter("method")),
        };

        // 添加 headers
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        // 添加 body
        if let Some(body) = &config.body {
            request = request.json(body);
        }

        let response = request.send().await?;
        let status = response.status();
        let body: Value = response.json().await.unwrap_or(json!({}));

        let output = ExecutionData::new_json(json!({
            "status": status.as_u16(),
            "body": body,
        }), None);

        Ok(make_execution_data_map(vec![
            (NodeConnectionKind::Main, vec![ExecutionDataItems::new_item(output)])
        ]))
    }
}
```

## Node Registration

```rust
pub fn register_all_nodes(registry: &NodeRegistry) -> Result<(), RegistrationError> {
    // Trigger nodes
    register_trigger_nodes(registry)?;

    // Core nodes
    register_core_nodes(registry)?;

    // LM nodes
    register_lm_nodes(registry)?;

    // Store nodes
    register_store_nodes(registry)?;

    // Integration nodes
    register_integration_nodes(registry)?;

    Ok(())
}

fn register_trigger_nodes(registry: &NodeRegistry) -> Result<(), RegistrationError> {
    let node = Arc::new(ManualTriggerNode::new()?);
    registry.register_node(node)?;

    let node = Arc::new(ScheduleTriggerNode::new()?);
    registry.register_node(node)?;

    Ok(())
}
```

## Best Practices

1. **参数解析**: 使用 `parse_config` 方法统一解析节点参数
2. **错误处理**: 使用 `NodeExecutionError` 明确错误类型
3. **版本管理**: 每个节点可以有多个版本 (V1, V2, ...)
4. **测试**: 为每个节点编写单元测试

## Examples from Codebase

- `hetumind/hetumind-nodes/src/trigger/manual_trigger/mod.rs` - 手动触发器
- `hetumind/hetumind-nodes/src/lm/lm_openai/mod.rs` - OpenAI 节点
- `hetumind/hetumind-nodes/src/core/if_node/mod.rs` - If 条件节点
