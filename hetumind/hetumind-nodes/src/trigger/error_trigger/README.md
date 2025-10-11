# ErrorTriggerNode

## 概述

ErrorTriggerNode 是一个工作流触发器节点，用于在其他工作流执行失败时自动触发错误处理工作流。该节点实现了类似 n8n Error Trigger 的功能，为 hetumind 平台提供了强大的错误处理能力。

## 功能特性

### 核心功能
- **错误工作流触发**: 当其他工作流执行失败时自动触发错误处理流程
- **灵活的错误过滤**: 支持按工作流ID、节点名称、错误类型和严重级别进行过滤
- **手动测试模式**: 提供示例错误数据用于开发和测试
- **结构化错误数据**: 生成标准化的错误信息结构

### 配置选项
- **触发模式**:
  - `All Workflows`: 监听所有工作流错误
  - `Specific Workflows`: 监听指定工作流错误
  - `Internal Only`: 仅当前工作流内部错误

- **错误类型过滤**:
  - Node Execution: 节点执行错误
  - Timeout: 工作流超时错误
  - Resource Exhausted: 资源不足错误
  - External Service: 外部服务错误
  - Validation: 数据验证错误
  - Configuration: 配置错误

- **重试机制**:
  - 启用/禁用自动重试
  - 配置最大重试次数
  - 设置重试间隔

- **通知功能**:
  - 邮件通知
  - Slack通知
  - Webhook通知
  - 数据库记录

## 技术架构

### 数据结构

#### WorkflowErrorData
```rust
pub struct WorkflowErrorData {
    pub workflow: WorkflowErrorSource,      // 错误来源工作流信息
    pub execution: Option<ExecutionErrorInfo>, // 执行错误信息
    pub trigger: Option<TriggerErrorInfo>,   // 触发器错误信息
}
```

#### 错误信息结构
- `WorkflowErrorSource`: 工作流基本信息 (ID, 名称)
- `ExecutionErrorInfo`: 执行上下文错误信息
- `TriggerErrorInfo`: 触发器错误信息
- `ErrorInfo`: 标准化错误详情

### 实现模式

ErrorTriggerNode 遵循 hetumind 标准节点实现模式：

1. **三文件结构**:
   - `mod.rs`: 核心节点结构和执行逻辑
   - `parameters.rs`: 配置参数解析和验证
   - `utils.rs`: 工具函数和基础配置构建

2. **版本管理**:
   - 当前版本: V1 (1.0.0)
   - 支持多版本执行器

3. **异步执行**:
   - 使用 `async_trait` 实现异步执行接口
   - 线程安全的 `Arc<NodeDefinition>` 设计

## 使用方法

### 基础配置示例

```rust
// 创建错误触发器节点
let error_node = ErrorTriggerNode::new()?;

// 注册到节点注册表
registry.register_node(Arc::new(error_node))?;
```

### 工作流配置

在 hetumind Studio 中配置 ErrorTriggerNode：

1. 添加 Error Trigger 节点到工作流
2. 配置触发模式和工作流过滤
3. 设置错误类型和严重级别
4. 配置重试和通知选项

### 手动测试

在手动执行模式下，节点会生成示例错误数据：

```json
{
  "workflow": {
    "id": "example-workflow-123",
    "name": "Example Workflow"
  },
  "execution": {
    "id": "execution-456",
    "url": "/workflow/execution/456",
    "retry_of": null,
    "error": {
      "message": "Example error message for testing",
      "stack": "at Node.execute (/path/to/node.js:42:15)",
      "name": "NodeExecutionError",
      "description": "This is a sample error for manual testing",
      "timestamp": "2024-01-15T10:30:00Z"
    },
    "last_node_executed": "failing-node",
    "mode": "manual"
  }
}
```

## 集成架构

### 工作流引擎扩展

ErrorTriggerNode 与 hetumind 工作流引擎深度集成：

```rust
pub trait WorkflowEngine {
    async fn execute_error_workflow(
        &self,
        error_data: WorkflowErrorData,
        error_workflow_id: Option<WorkflowId>,
    ) -> Result<ExecutionResult, WorkflowExecutionError>;
}
```

### 错误处理流程

1. **错误检测**: 工作流执行失败时检测错误
2. **数据构建**: 创建标准化的错误数据结构
3. **工作流查找**: 查找配置的错误工作流或包含 ErrorTrigger 的工作流
4. **权限验证**: 检查跨工作流调用权限
5. **执行触发**: 执行错误工作流并传递错误数据

## 安全机制

### 防无限循环
- 检查错误工作流是否形成递归调用
- 防止错误工作流自身触发错误处理

### 权限控制
- 验证工作流所有权
- 检查跨工作流调用权限
- 防止未授权的错误工作流执行

## 测试

项目包含完整的单元测试：

```bash
cargo test -p hetumind-nodes --test test_error_trigger
```

测试覆盖：
- 节点注册和创建
- 属性定义验证
- 手动执行模式
- 配置参数解析

## 依赖关系

### 核心依赖
- `hetumind-core`: 工作流核心类型和接口
- `async-trait`: 异步特征支持
- `serde`: 序列化/反序列化

### 可选功能
- 通知系统集成
- 重试机制
- 错误分类和过滤

## 扩展性

ErrorTriggerNode 设计为可扩展架构：

1. **新错误类型**: 在 `ErrorType` 枚举中添加新类型
2. **通知渠道**: 扩展 `NotificationMethod` 枚举
3. **过滤条件**: 添加新的配置参数
4. **数据格式**: 扩展 `WorkflowErrorData` 结构

## 最佳实践

### 错误工作流设计
- 使用 Error Trigger 作为起始节点
- 包含适当的错误分类逻辑
- 实现通知和恢复机制
- 记录错误处理历史

### 配置建议
- 为关键工作流配置专用错误处理
- 根据业务重要性设置错误严重级别
- 合理配置重试策略避免资源浪费
- 设置适当的通知阈值

## 版本历史

- **v1.0.0** (初始版本)
  - 基础错误触发功能
  - 灵活的配置选项
  - 手动测试模式
  - 完整的测试覆盖