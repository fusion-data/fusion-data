# hetuflow-core

`hetuflow-core` 是 hetuflow 分布式任务调度系统的共享核心库。作为系统的基础设施层，它定义了 Agent 与 Server 之间的通信协议、数据模型和类型规范，确保整个系统的一致性和类型安全。

## 功能特性

- **完整的通信协议定义**：基于 WebSocket 的双向实时通信
- **类型安全**：使用 Rust 的强类型系统确保编译时类型检查
- **模块化设计**：清晰的模块分离，易于维护和扩展
- **序列化支持**：基于 Serde 的 JSON 序列化/反序列化
- **数据库支持**：可选的 SQLx 集成，支持 PostgreSQL

## 模块结构

```
hetuflow-core/
├── src/
│   ├── lib.rs          # 库入口和公共API
│   ├── types.rs        # 基础类型定义和枚举
│   ├── models.rs       # 共享数据模型
│   └── protocol.rs     # WebSocket消息协议定义
├── examples/
│   └── basic_usage.rs  # 使用示例
└── Cargo.toml          # 依赖配置
```

## 核心类型

### 1. 枚举类型 (types.rs)

- `ScheduleKind`: 作业调度类型 (Cron, Time, Daemon, Event, Flow)
- `TaskInstanceStatus`: 任务执行状态
- `AgentStatus`: Agent 状态
- `AgentCommandKind`: Agent 指令类型
- `TaskControlKind`: 任务控制类型
- `MessageKind`: WebSocket 消息类型
- `WebSocketError`: WebSocket 错误类型

### 2. 数据模型 (models.rs)

- `AgentCapabilities`: Agent 能力描述
- `AgentConfig`: Agent 配置信息
- `TaskConfig`: 任务配置
- `AgentMetrics`: Agent 性能指标
- `TaskMetrics`: 任务执行指标
- `TaskStatusInfo`: 任务状态信息

### 3. 协议消息 (protocol.rs)

- `WebSocketMessage`: 统一消息包装器
- `AgentRegisterRequest/Response`: Agent 注册消息
- `HeartbeatRequest/Response`: 心跳消息
- `DispatchTaskRequest/Response`: 任务分发消息
- `TaskInstanceUpdate`: 任务状态更新
- `TaskControl`: 任务控制指令

## 快速开始

### 1. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
hetuflow-core = { path = "../fusion/hetuflow-core", features = ["with-db"] }
```

### 2. 基本使用示例

```rust
use hetuflow_core::*;
use std::collections::HashMap;

// 创建Agent注册请求
let capabilities = AgentCapabilities {
    max_concurrent_tasks: 10,
    supported_task_types: vec!["shell".to_string(), "python".to_string()],
    resources: {
        let mut map = HashMap::default();
        map.insert("cpu".to_string(), "8 cores".to_string());
        map.insert("memory".to_string(), "16GB".to_string());
        map
    },
    features: vec!["docker".to_string(), "gpu".to_string()],
};

let register_request = AgentRegisterRequest {
    agent_id: "agent-001".to_string(),
    namespace_id: "production".to_string(),
    capabilities,
    metadata: HashMap::default(),
    version: "1.0.0".to_string(),
    hostname: "server-01".to_string(),
    os_info: "Linux Ubuntu 22.04".to_string(),
};

// 创建WebSocket消息
let message = WebSocketMessage::new(
    MessageKind::AgentRegister,
    serde_json::to_value(&register_request).unwrap(),
);
```

### 3. 运行示例

```bash
cargo run --example basic_usage --all-features
```

## 特性标志

- `with-db`: 启用数据库支持（SQLx 集成）

## 技术栈

- **异步运行时**: Tokio
- **序列化**: Serde + JSON
- **UUID**: uuid crate
- **时间处理**: chrono
- **错误处理**: thiserror
- **数据库**: SQLx (可选)

## 许可证

本项目采用 Apache-2.0 许可证，详见 [LICENSE.txt](../../LICENSE.txt)。
