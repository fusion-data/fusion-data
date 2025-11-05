# Hetumind 插件机制（WASM）方案摘要

## 说明与范围

本次重构不涉及 WASM 插件功能的实现，仍以 hetumind 内置节点为主。本摘要用于明确后续支持社区/个人插件的总体技术路线和接口约定，便于未来迭代。

## 设计目标

- 安全沙箱：插件运行在 WASM 环境中，具备资源配额与受限能力（CPU、内存、文件系统/网络隔离）。
- 跨平台分发：插件以单文件 .wasm + manifest 分发，宿主跨平台加载。
- 可观测性：统一纳入引擎 Trace/Metrics，便于观测/调试/重试。
- 开发者体验：提供 SDK（Rust/TS），简化插件开发与测试。

## 插件打包与清单（Manifest）

- 文件结构：
  - manifest.json/toml：插件元信息与节点定义宣告
  - plugin.wasm：WASM 二进制
- manifest 字段建议：
  - name/display_name/author/license/version
  - host_api_version（宿主 API 版本兼容性）
  - node_kinds: [{ kind, versions, type: executor|supplier, provider_type: llm|memory|tool|agent }]
  - io_ports: inputs/outputs（使用 ConnectionKind 名称）
  - properties: NodeProperty 简要声明（用于 UI 与 Schema 生成）
  - compatibility/constraints：平台/宿主版本约束

示例（JSON）：

```json
  {
    "name": "community.deepseek",
    "version": "0.1.0",
    "host_api_version": "v1",
    "node_kinds": [
      { "kind": "ai.deepseek.model", "versions": ["1.0.0"], "type": "supplier", "provider_type": "llm" }
    ],
    "io_ports": {
      "inputs": ["main"],
      "outputs": ["ai_lm", "error"]
    },
    "properties": [
      { "name": "model", "kind": "string", "required": true }
    ]
  }
```

## 宿主-插件 API（WIT 接口约定）

- 标准导出函数（插件 → 宿主调用）：
  - init() → Result<(), Error>
  - get_node_definitions() → [NodeDefinitionSerializable]
  - create_executor(kind, version) → ExecutorHandle
  - create_supplier(kind, version) → SupplierHandle
  - execute(handle, input_json, ctx_api) → output_json
  - supply(handle, ctx_api) → data_json
- ctx_api 能力（宿主 → 插件提供受限 API）：
  - 日志/计时/随机数
  - 表达式计算、HTTP 请求代理、二进制存储引用
  - Memory Service 访问（受限、鉴权、租户隔离）

## Registry 与 Loader（宿主侧）

- 扩展 NodeRegistry：
  - plugin_nodes: Map<NodeKind, PluginExecutorFactory>
  - plugin_providers: Map<NodeKind, PluginSupplierFactory>
  - get_executor/get_supplier 时优先静态，缺失则回退到插件工厂创建包装器
- DirectoryLoader：
  - 扫描 plugins/ 目录 → 校验 manifest → 加载 wasm → 调用 init/get_node_definitions → 注册
  - 支持热重载（文件变更触发卸载/重载，需谨慎处理正在运行的执行器）

## 上下文桥接与安全

- 多租户隔离：ctx_api 自动注入 tenant_id 与 workflow_id；插件侧不可绕过宿主 IAM。
- 资源限制：WASM 运行时配置配额；网络与文件访问通过宿主代理。
- 错误治理：统一错误语义（EngineError/NodeExecutionError），可观测性一致。

## 迭代路线（不在本次重构范围内）

1) 第一阶段：
  - 保留静态节点体系；支持通过 WASM 加载插件的“定义 + 执行”。
  - NodeRegistry 增加 plugin_* 存储与包装器。
2) 第二阶段：
  - 提供开发者 SDK 与示例插件；完善 ctx_api 能力与测试框架。
3) 第三阶段：
  - 类型安全的 Registry 与更严格的 Schema 校验；插件仓库与签名校验。

## 与 n8n 的对齐

- 类似 n8n 的 DirectoryLoader 与动态加载模型，但以 WASM 提供强隔离与跨语言能力。
- 节点定义/端口/属性与 hetumind 的 NodeDefinition/ConnectionKind/NodeProperty 映射一致。

—

备注：本方案为摘要说明，实际实现需在 hetumind-core/studio 引擎层补充 Registry/Loader/WIT/ctx_api 等组件，并提供安全策略与测试规范。