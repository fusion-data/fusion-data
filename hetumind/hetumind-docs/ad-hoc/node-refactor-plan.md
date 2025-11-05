# Hetumind Node 重构方案（对齐 n8n 架构并复用现有代码）

## 目标与背景

基于《n8n Node 设计架构分析报告》，结合当前 hetumind 已实现的核心与节点代码，制定一套可落地的 Node 重构方案，重点对齐以下能力：

- 统一的节点注册与版本管理（NodeRegistry / Node + FlowNodeRef / SubNodeRef）
- 连接类型驱动的组件发现（AiLM / AiMemory / AiTool / AiAgent 等）
- 执行上下文 API（获取上游连接数据、参数解析、缓存优化）
- Sub Node Provider 供给机制（LLM/Memory/Tool/Agent）
- 动态工具包装（将任意节点转换为 Agent 可调用的工具）

本方案要求在不引入大规模破坏性改造的前提下，最大化复用现有代码，并逐步补齐缺失的接口与实现。

## 与 n8n 架构的对齐点

1) 节点创建与注册

- 现状：已具备 NodeRegistry、Node trait（聚合多版本 FlowNode 执行器 + SubNode 供应器）、注册方法（register_node/register_subnode_provider）
- 方案：沿用现有 NodeRegistry 能力，确保每个功能模块在 mod.rs 内集中注册，同时为 AI 相关节点补齐 SubNode Provider 的注册（与 n8n 的“节点类 + 版本选择 + 供给”一致）。

2) 连接类型驱动

- 现状：已有 ConnectionKind（Main/Error/AiAgent/AiTool/AiLM/AiMemory/...），与 n8n 的 NodeConnectionTypes 高度对齐
- 方案：标准化 LLM/Memory/Tool/Agent 的输出端口含义：
  - AiLM：输出“模型实例/模型配置/模型能力”
  - AiMemory：输出“会话内存/最近消息窗口/上下文键配置”
  - AiTool：输出“工具实例或工具集合（Toolkit 展开）”
  - AiAgent：输出“代理执行器/中间请求（EngineRequest）/最终结果”

3) 执行上下文 API

- 现状：NodeExecutionContext 已支持：
  - get_connection_data/get_all_connections_data/get_all_connections（基于 ConnectionKind 的数据获取）
  - get_input_data/get_parameters/get_node_parameter（参数读取/校验）
  - 连接管理器（hetumind-nodes/src/core/connection_manager.rs）提供可复用的缓存优化 API
- 方案：引入“语义化”的子节点发现 Helper（在 hetumind-core 或 hetumind-nodes 公共模块中提供）：
  - get_llm_providers(context, index) → Vec<LLMSubNodeProviderRef>
  - get_memory_provider(context, index) → Option<MemorySubNodeProviderRef>
  - get_connected_tools(context) → Vec<ToolSubNodeProviderRef>
  - 注意：这些 Helper 不改变 NodeExecutionContext 现有结构，只在 helper 内部使用 workflow.connections + NodeRegistry + 上游 NodeElement.parameters 完成映射与实例化。

4) Sub Node Provider 供给机制

- 现状：已定义 trait（SubNode/LLMSubNodeProvider/MemorySubNodeProvider/ToolSubNodeProvider/AgentSubNodeProvider），但注册与具体实现尚不完整
- 方案：为 LLM/Memory/Tool 节点补齐对应 Provider，实现“supplyData”式的供给接口：
  - DeepSeek（LLM）：提供 LLMSubNodeProvider，输出 rig-core Agent/Client 或可调用模型句柄
  - SimpleMemory：提供 MemorySubNodeProvider，封装滑动窗口内存（优先工作流级别存储）
  - Tool 节点（后续新增，如 Wikipedia/HTTP）：提供 ToolSubNodeProvider，支持单工具与 Toolkit 展开
  - Agent 节点：提供 AgentSubNodeProvider（或直接实现 FlowNode 的代理执行），统一调用模型 + 内存 + 工具

5) 动态工具包装（create_node_as_tool）

- 现状：NodeProperty 类型已具备大量 UI/校验元数据；可通过附加属性生成简化的参数 Schema
- 方案：在 hetumind-core 增加工具包装函数，将任意节点（尤其数据处理类节点）转换为 Agent 可调用的工具：
  - 从 NodeDefinition.properties 中收集标记为“from_ai”的参数（使用 NodeProperty.additional_properties.from_ai = true）
  - 基于 NodeProperty.kind / validate_type / options 生成简化 JSON Schema
  - 构建 ToolSubNodeProvider 的 as_tool() 返回值（名称、描述、schema、func），其中 func 内部调用该节点的 execute，并处理输入/输出转换

## 现有代码的复用与改造点

1) NodeRegistry（hetumind-core/src/workflow/node_registry.rs）

- 已支持：
  - register_node/get_executor/get_definition
  - register_subnode_provider/get_subnode_provider（可直接用于 Provider 注册与发现）
- 改造点：
  - 在各节点模块的 register_nodes 中，同时注册 Executor 与 Provider（如 DeepseekModelNode + DeepseekModelSupplier）
  - 为 Agent 相关模块增加 Provider 注册，以便上下文检索

2) NodeDefinition（hetumind-core/src/workflow/node.rs + port.rs）

- 已支持：
  - 输入/输出端口声明（InputPortConfig/OutputPortConfig）
  - 属性元数据（NodeProperty + NodePropertyKind）
- 改造点：
  - 约定 AI 场景的端口声明：LLM/Memory/Tool/Agent 的输入/输出均使用 ConnectionKind 对齐 n8n
  - 使用 NodeProperty.additional_properties 标记 from_ai、ai_key 等，便于工具模式的参数收集

3) FlowNode（hetumind-core/src/workflow/flow_node.rs）

- 已支持：
  - init/execute/definition 基础接口
- 改造点：
  - 保持不变，Agent 等复杂节点以 FlowNode 形式实现执行逻辑；“供给”通过 Provider 完成（避免在 FlowNode 中夹杂对象实例化）

4) SubNode Provider（hetumind-core/src/workflow/sub_node.rs）

- 已支持：
  - SubNode/LLMSubNodeProvider/MemorySubNodeProvider/ToolSubNodeProvider/AgentSubNodeProvider 接口
  - LLMConfig/MemoryConfig/ToolConfig/AgentConfig/Message 等模型
- 改造点：
  - 在 SubNode trait 中增加 as_any() 以支持 downcast（或通过 provider_type 做类型分派），用于 Agent 侧进行具体 Provider 能力调用
  - 在 Provider 实现中读取上游 NodeElement.parameters，构建对应的配置（避免修改 ExecutionData 的结构）

5) 连接数据管理器（hetumind-nodes/src/core/connection_manager.rs）

- 已支持：
  - get_connection_data_optimized/get_all_connections_optimized（带缓存）
- 改造点：
  - 在 Helper 中复用该能力，统一在 Agent 执行流中进行数据检索与缓存

6) 已有节点模块示例

- IfNode（hetumind-nodes/src/core/if_node/）：作为标准 FlowNode 执行器的代表，保持现状
- DeepSeek（hetumind-nodes/src/llm/deepseek_node/）：
  - 现状：DeepseekV1 FlowNode 仅输出 AiLM 执行数据（模型信息/能力），尚未提供 Provider
  - 方案：新增 DeepseekModelSupplier 实现 LLMSubNodeProvider，注册到 NodeRegistry.subnode_providers
    - Supplier 在 as_tool/或 call_llm 中创建 rig-core Agent/Client；支持 messages + LLMConfig
    - 配置读取优先使用上游 NodeElement.parameters（model/api_key/max_tokens/temperature 等）
- SimpleMemory（hetumind-nodes/src/memory/simple_memory_node/）：
  - 现状：FlowNode 输出 AiMemory，但未有工作流级内存存储
  - 方案：新增 SimpleMemorySupplier，实现 MemorySubNodeProvider
    - 将 WorkflowMemoryBuffer 改为 ExecutionContext 持久化（会话期间），通过 session_id 读写
    - FlowNode 与 Supplier 共享同一配置结构 SimpleMemoryConfig

## 关键接口与示例（建议实现草案）

1) 语义化子节点发现 Helper（以 LLM 为例）

```rust
  /// 从当前节点的上游连接中检索 LLM 供应器（按连接逆序）
  /// - 复用 NodeExecutionContext.get_all_connections / NodeRegistry.get_subnode_provider
  pub async fn get_llm_providers(
    ctx: &NodeExecutionContext,
    index: usize,
  ) -> Result<Vec<LLMSubNodeProviderRef>, NodeExecutionError> {
    use crate::workflow::{ConnectionKind, NodeKind};

    let parents = ctx.get_all_connections(&ConnectionKind::AiLM);
    let mut providers = Vec::new();

    for (i, conn) in parents.iter().rev().enumerate() {
      if i < index { continue; }
      let node = ctx.workflow.nodes.iter().find(|n| n.name == conn.node_name)
        .ok_or_else(|| NodeExecutionError::NodeNotFound {
          workflow_id: ctx.workflow.id.clone(),
          node_name: conn.node_name.clone(),
        })?;

      if let Some(p) = ctx.node_registry.get_subnode_provider(&node.kind) {
        // 运行时按 provider_type 检查并 downcast 到 LLMSubNodeProviderRef（可通过 as_any 辅助实现）
        // 这里省略具体 downcast 代码，方案阶段保持接口约定
        providers.push(p.clone() as LLMSubNodeProviderRef);
      }
    }
    Ok(providers)
  }
```

2) DeepSeek 供应器（LLMSubNodeProvider）

```rust
  /// DeepSeek LLM 供应器示例（函数级注释已添加）
  pub struct DeepseekModelSupplier {
    definition: Arc<NodeDefinition>,
  }

  #[async_trait]
  impl SubNode for DeepseekModelSupplier {
    fn provider_type(&self) -> SubNodeType { SubNodeType::LLM }
    fn definition(&self) -> Arc<NodeDefinition> { self.definition.clone() }
    async fn initialize(&self) -> Result<(), NodeExecutionError> { Ok(()) }
  }

  #[async_trait]
  impl LLMSubNodeProvider for DeepseekModelSupplier {
    /// 调用 LLM，内部基于 rig-core 构建 Agent/Client
    async fn call_llm(
      &self,
      messages: Vec<Message>,
      config: LLMConfig,
    ) -> Result<LLMResponse, NodeExecutionError> {
      // 读取 api_key 并创建 deepseek 客户端
      let client = rig::providers::deepseek::Client::new(config.api_key.as_deref().unwrap_or(""));
      let mut ab = client.agent(&config.model);
      if let Some(temp) = config.temperature { /* 绑定必要的参数 */ }
      let agent = ab.build();

      // 将 Message 转为 rig::message::Message 并调用
      // 返回标准化的 LLMResponse（content/role/usage）
      // 细节与 DeepseekV1.execute 中保持一致，避免重复逻辑
      todo!("实现调用与响应封装")
    }
  }
```

3) 动态工具包装（create_node_as_tool）

```rust
  /// 将任意 FlowNode 包装为 Tool，供 Agent 调用
  pub fn create_node_as_tool(
    node_name: &str,
    node_def: &NodeDefinition,
    handle_tool_invocation: impl Fn(serde_json::Value) -> Result<serde_json::Value, NodeExecutionError> + Send + Sync + 'static,
  ) -> Tool {
    // 基于 NodeProperty 收集 additional_properties.from_ai=true 的参数，生成简化 JSON Schema
    // 规则：
    // - name 校验：^[a-zA-Z0-9_-]{1,64}$
    // - kind → schema 类型映射（String/Number/Boolean/Json）
    // - validate_type → 强化类型校验
    // - multiple_values → 生成数组类型
    // 生成 Tool {name, description, parameters(schema)}，func 内部调用 handle_tool_invocation
    todo!("Schema 生成与 Tool 构造")
  }
```

4) Agent 执行流程（ToolsAgentV1 草案）

```rust
  /// Agent 节点执行：发现 LLM/Memory/Tool，构建执行器并按需返回工具调用请求
  #[async_trait]
  impl FlowNode for ToolsAgentV1 {
    async fn execute(&self, ctx: &NodeExecutionContext) -> Result<ExecutionDataMap, NodeExecutionError> {
      // 1. 解析输入（prompt/steps/选项）
      let input = ctx.get_input_data(ConnectionKind::Main)?;

      // 2. 发现与构建依赖
      let llms = get_llm_providers(ctx, 0).await?;
      let memory = get_memory_provider(ctx, 0).await?; // 可选
      let tools = get_connected_tools(ctx).await?;

      // 3. 创建代理执行器（伪代码）
      let executor = create_agent_sequence(llms.first(), tools, /*prompt/options*/);

      // 4. 执行代理，并根据结果：
      // - 若产生工具调用：在 AiTool 端口返回 EngineRequest 风格的数据项
      // - 否则：在 Main 端口返回最终回答
      todo!("Agent 调度与结果封装")
    }
  }
```

## 数据流与执行步骤（对齐 n8n）

- 初始化：各节点模块在系统启动阶段注册 Executor 与 Provider 到 NodeRegistry
- 工作流配置：NodeDefinition 声明输入/输出端口与属性；Workflow JSON 配置好节点与连接关系
- 执行上下文：DefaultWorkflowEngine 基于图收集父节点输出，构建 NodeExecutionContext
- 节点执行：
  - 普通节点（如 If）：直接 execute 返回 Main/Error 等输出
  - Sub Node（LLM/Memory/Tool）：FlowNode 可选择仅做“供给数据”的输出；Agent 通过 Provider 接口获取并使用运行时对象
- Agent 调度：
  - 通过 Helper 查找 AiLM/AiMemory/AiTool 的上游 Provider
  - 创建代理执行器，必要时创建工具调用请求（EngineRequest 风格）
  - 最终返回回答或继续发起工具调用

## 版本与注册示例

以 DeepSeek 模块为例（现有 mod.rs 仅注册 Executor）：

```rust
  pub struct DeepseekModelNode {
    default_version: Version,
    executors: Vec<FlowNodeRef>,
    suppliers: Vec<SubNodeRef>,
  }

  impl DeepseekModelNode {
    pub fn new() -> Result<Self, RegistrationError> {
      let executors: Vec<FlowNodeRef> = vec![Arc::new(DeepseekV1::new()?)];
      let suppliers: Vec<SubNodeRef> = vec![Arc::new(DeepseekModelSupplier::new()? )];
      let default_version = executors.iter().map(|n| n.definition().version.clone()).max().unwrap();
      Ok(Self { default_version, executors, suppliers })
    }
  }

  impl Node for DeepseekModelNode {
    fn default_version(&self) -> &Version { &self.default_version }
    fn node_executors(&self) -> &[FlowNodeRef] { &self.executors }
    fn node_suppliers(&self) -> &[SubNodeRef] { &self.suppliers }
    fn kind(&self) -> NodeKind { self.executors[0].definition().kind.clone() }
  }

  pub fn register_nodes(registry: &NodeRegistry) -> Result<(), RegistrationError> {
    let node = Arc::new(DeepseekModelNode::new()?);
    registry.register_node(node.clone())?;
    // 注册 Provider（也可通过 register_subnode_provider 单独注册）
    for s in node.node_suppliers() {
      registry.register_subnode_provider(node.kind(), s.clone())?;
    }
    Ok(())
  }
```

## 最小改动清单（建议）

- 在 hetumind-nodes 的 DeepSeek 与 SimpleMemory 模块中新增各自的 Supplier 实现与注册
- 在 hetumind-core 增加 create_node_as_tool（动态工具包装）与简化 Schema 生成工具
- 在 hetumind-nodes 增加 Helper（get_llm_providers/get_memory_provider/get_connected_tools），内部使用 NodeExecutionContext + NodeRegistry + connection_manager
- 在 Application 组件系统中注入独立 Memory Service（支持 Redis/Valkey 或本地内存实现），供 Memory Supplier/FlowNode 使用（多租户隔离 + TTL 清理）
- 在 SubNode trait 中（或其实现）增加 as_any/downcast 能力，便于 Agent 对具体 Provider 调用（可选项，亦可通过 provider_type 进行模式匹配）

## 兼容性与边界

- 不引入审计/迁移逻辑（遵循 CLAUDE.md）
- 不改变现有 FlowNode execute 签名与基本行为；Agent 的扩展通过 Provider + Helper 实现
- ConnectionKind 作为统一连接类型，不新增破坏性枚举项；必要扩展仅在 NodeDefinition.properties 上进行

## 已确认决策（合并入方案）

1) LLM 的运行时对象选择：
  - 已确认统一使用 rig-core 的 Agent/Client 作为标准模型句柄。
  - 影响：所有 LLMSubNodeProvider 及相关执行逻辑均以 rig-core 提供的 Agent/Client 为唯一运行时对象，避免多套适配层，降低复杂度，增强一致性。

2) 动态工具参数 Schema：
  - 已确认接受“简化版 JSON Schema”（由 NodeProperty 推导）作为第一阶段产物。
  - 影响：create_node_as_tool 的 Schema 生成将以 NodeProperty.kind / validate_type / options 为依据，生成可用于基本校验的简化 Schema；后续可迭代接入更严格的校验库（schemars/valico）。

3) Memory 的工作流级存储：
  - 已确认采用方案 C：独立 Memory Service 组件（在 Application 组件系统中注入）。
  - 设计要点：
    - 以独立服务组件形式注入到 Application（例如通过 Application::global().component 获取），由宿主统一管理生命周期与资源。
    - 后端可插拔：支持 Redis/Valkey、本地内存 + TTL；支持跨执行共享与持久会话（按租户/工作流/会话维度隔离）。
    - 多租户隔离策略：Memory Service API 自动注入 tenant_id 与 workflow_id；提供会话键生成策略（workflow_id + session_id）。
    - 统一接口：get_buffer/store_messages/retrieve_messages/cleanup，供 SimpleMemorySupplier/FlowNode 使用。
    - 与 NodeExecutionContext 的关系：上下文仅持有到 Memory Service 的引用，不负责内存存储；执行引擎负责清理策略与观测指标整合。

4) Agent 的工具调用返回：
  - 已确认采用方案 1：EngineRequest 风格（在 AiTool 端口输出请求对象，由引擎调度二次执行）。
  - 设计要点：
    - 数据模型：定义 EngineRequest/EngineResponse 结构（包含 nodeName、type=AiTool、id、input、metadata），与 ConnectionKind::AiTool 对齐。
    - 调度路径：默认由工作流引擎解析 EngineRequest，路由到对应的 Tool 节点进行执行，返回 EngineResponse 或最终结果。
    - 观测性：所有工具调用进入统一执行路径，便于 Trace/Metrics/重试策略；支持并发/串联/条件执行。
    - 兼容性：保留简单场景下 Agent 内部直接调用工具的 fallback（仅用于 PoC 或单工具），但生产场景统一采用 EngineRequest。

## 待确认问题（请回答或选择更优方案）

5) Provider 的 downcast 能力：
  - 方案建议通过 as_any + Any/downcast 实现，是否同意该设计？若不同意，可通过 provider_type 模式匹配 + trait object 切分接口实现。

—

如以上方案与现状相符，我将基于该设计在 DeepSeek 与 SimpleMemory 两个模块率先补齐 Provider 与注册，随后补充工具包装与 Agent 流程的 Helper 实现。若你倾向于不同的运行时对象或工具 Schema 方案，请先确认第 1/2 点，以便实施时避免返工。

## 合并的优化项（已纳入实施）

- 规范 EngineRequest/Response 数据结构与错误语义（含重试、幂等、correlation_id），在 hetumind-core 统一定义并被 Agent/Tool 复用
- 为 Registry 增加 typed 获取接口（get_llm_supplier/get_memory_supplier/get_tool_supplier/get_agent_supplier），减少运行时 downcast 使用频率
- 简化 Schema 校验采用 jsonschema crate（https://crates.io/crates/jsonschema），UI 与引擎侧共享同一 schema 保证一致性
- Memory Service 后端抽象：定义 MemoryBackend（InMemory/Redis/Hybrid），按租户/工作流策略选择后端与缓存层次，优化性能与成本
- 执行上下文与权限：在 ctx_api 中注入租户上下文与权限校验，确保 Memory/Tool/LLM 等调用遵循 IAM 规则（参考 Jieyuan）

## 编码任务清单（AI Coding Prompt）

请按以下任务顺序实现（无需包含周期安排）：

1) Memory Service 组件（独立注入 Application）
  - 文件建议：hetumind-context/src/services/memory_service.rs
  - 接口定义（函数级注释必须）：
  ```rust
    /// 内存服务后端抽象，支持多租户隔离与持久化后端
    pub trait MemoryService {
      /// 获取或创建会话缓冲区
      fn get_buffer(&self, tenant_id: &str, workflow_id: &str, session_id: &str) -> Result<WorkflowMemoryBuffer, NodeExecutionError>;

      /// 追加存储消息
      fn store_messages(&self, tenant_id: &str, workflow_id: &str, session_id: &str, messages: Vec<Message>) -> Result<(), NodeExecutionError>;

      /// 检索最近 N 条消息
      fn retrieve_messages(&self, tenant_id: &str, workflow_id: &str, session_id: &str, count: usize) -> Result<Vec<Message>, NodeExecutionError>;

      /// 清理过期会话，返回统计信息
      fn cleanup(&self, expired_before: chrono::DateTime<chrono::Utc>) -> Result<serde_json::Value, NodeExecutionError>;
    }
  ```
  - 后端实现建议：
    - InMemoryMemoryService（本地内存 + TTL）
    - RedisMemoryService（基于 Redis/Valkey，键格式：{tenant_id}:{workflow_id}:{session_id}）
  - Application 注入：通过 fusion_core::application::Application 注册为全局组件，并在 NodeExecutionContext 中以引用方式访问。

2) EngineRequest/Response 统一模型
  - 文件建议：hetumind-core/src/workflow/engine_request.rs
  - 结构草案：
  ```rust
    /// 工具调用请求（由 Agent 节点输出到 AiTool 端口）
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EngineRequest {
      pub node_name: String,
      pub r#type: ConnectionKind, // 固定为 AiTool
      pub id: String,
      pub input: serde_json::Value,
      pub metadata: serde_json::Value,
      pub correlation_id: Option<String>,
      pub retry_policy: Option<RetryPolicy>,
    }

    /// 工具调用响应（由引擎或 Tool 节点返回）
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EngineResponse {
      pub id: String,
      pub output: serde_json::Value,
      pub error: Option<String>,
      pub correlation_id: Option<String>,
    }
  ```

3) Registry typed 接口增强
  - 在 hetumind-core/src/workflow/node_registry.rs 增加：
  ```rust
    /// 获取指定 NodeKind 的 LLM Supplier（若存在）
    pub fn get_llm_supplier(&self, kind: &NodeKind) -> Option<LLMSubNodeProviderRef> { /* typed 提取 */ }
    /// Memory/Tool/Agent 同理
  ```

4) create_node_as_tool 与 JSON Schema 校验
  - 文件建议：hetumind-core/src/workflow/tooling.rs
  - 要点：
    - 从 NodeDefinition.properties 中收集 additional_properties.from_ai=true 的参数，生成简化 JSON Schema
    - 使用 jsonschema crate 做运行时校验：当 Agent 传入工具调用参数时先校验，失败则返回结构化错误
  - 代码片段：
  ```rust
    /// 根据 NodeDefinition 生成简化 JSON Schema，并返回可用于校验的编译器
    pub fn compile_tool_schema(def: &NodeDefinition) -> Result<jsonschema::CompiledSchema, NodeExecutionError> {
      let schema = build_json_schema_from_properties(&def.properties);
      let compiled = jsonschema::JSONSchema::compile(&schema).map_err(|e| NodeExecutionError::ConfigurationError(e.to_string()))?;
      Ok(compiled)
    }
  ```

5) Helpers（hetumind-nodes 公共模块）
  - 文件建议：hetumind-nodes/src/common/helpers.rs
  - 提供：get_llm_providers/get_memory_provider/get_connected_tools 三个方法，内部复用 NodeExecutionContext + NodeRegistry + connection_manager。

6) SimpleMemorySupplier 对接 Memory Service
  - 修改 hetumind-nodes/src/memory/simple_memory_node/，使其通过 MemoryService 读写会话缓冲，不再在节点内部维护全局 Map。

7) DeepSeekModelSupplier（LLM）
  - 在 hetumind-nodes/src/llm/deepseek_node/ 新增 Supplier，实现 LLMSubNodeProvider，使用 rig-core Agent/Client，并支持 api_key 解析（resolve_api_key）。

8) 测试与验收
  - 单元测试：
    - MemoryService 后端：隔离性、TTL 清理、并发安全
    - EngineRequest/Response：序列化/反序列化、一致性
    - JSON Schema：工具参数校验，错误信息覆盖率
  - 集成测试：
    - Agent → Tool 调用链路（EngineRequest 风格），多工具并发/串联
    - LLM + Memory 联动：会话内持续上下文（跨执行共享）

## 约束与实现规范（用于 Prompt）

- 代码风格：Rust 2024，2 空格缩进，函数级注释必填，零 unsafe，遵循 CLAUDE.md 规则
- 连接类型：严格使用 ConnectionKind（Main/Error/AiAgent/AiTool/AiLM/AiMemory/...），禁止自定义字符串类型
- 参数模型：NodeProperty.additional_properties.from_ai=true 作为工具参数暴露的唯一入口；JSON Schema 校验使用 jsonschema crate
- 租户与权限：所有 Memory/Tool/LLM 调用均需通过上下文注入的租户信息，遵循 jieyuan IAM 约束（必要时拒绝缺失参数）
- 兼容性：不修改 FlowNode.execute 签名，不引入审计与迁移逻辑

## 术语与引用

- jsonschema crate：https://crates.io/crates/jsonschema
- rig-core（DeepSeek 等模型接入）：参考 hetumind-nodes/src/llm/deepseek_node/
- Application 组件系统：fusion_core::application::Application
- connection_manager：hetumind-nodes/src/core/connection_manager.rs