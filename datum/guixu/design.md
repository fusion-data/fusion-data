# guixu flow

## 技术

- WEB: Vue 3, opentiny
- BACKEND: Rust(Axum), Tokio, Sqlx(SeaQuery)
- DATABASE: PostgreSQL(pgvector)

MCP、Worker 节点 支持的运行工具

- Rust: cargo
- Node: pnpm (pnpm dlx)
- Python: uv

## 工作流 workflow

### 组件 components

#### 节点 nodes

工作流的第一个节点需要支持是 trigger 的 node

##### 节点操作：触发器和动作

当您将一个节点添加到工作流时，n8n 会显示可用的操作列表。操作是节点执行的一些任务，例如获取或发送数据。

有两种类型的操作：

- **trigger** 触发器会在响应服务中的特定事件或条件时启动工作流。选择触发器后，n8n 将向您的工作流中添加一个触发器节点，并预先选中您所选的触发器操作。在 n8n 中搜索节点时，触发器操作具有螺栓图标。
- **action** 动作代表工作流内的具体任务，您可以使用这些任务来处理数据、对外部系统进行操作以及作为工作流的一部分，在其他系统中触发事件。选择动作后，n8n 将向您的工作流中添加一个节点，并预先选中您所选的动作操作。

#### 连接器 connectors

#### 备注 notes
