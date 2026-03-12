# hetuflow-agent

任务执行代理：WebSocket 连接、任务执行、心跳上报。

## Imports

```rust
use hetuflow_agent::{
    Agent, AgentConfig, TaskExecutor,
    connection::WebSocketConnection,
    executor::{BashExecutor, PythonExecutor, NodeExecutor},
};
use hetuflow_core::protocol::{Command, Event, CommandKind, EventKind};
```

## Agent 架构

### Agent

```rust
use hetuflow_core::protocol::{Command, Event, EventKind};
use tokio_tungstenite::WebSocketStream;

pub struct Agent {
    id: Uuid,
    config: AgentConfig,
    connection: Option<WebSocketConnection>,
    executor: TaskExecutor,
    status: AgentStatus,
}

impl Agent {
    pub async fn connect(&mut self, server_url: &str) -> Result<()> {
        // 建立 WebSocket 连接
        let (ws_stream, _) = tokio_tungstenite::connect_async(server_url).await?;
        self.connection = Some(WebSocketConnection::new(ws_stream));

        // 发送注册事件
        self.send_event(Event {
            kind: EventKind::RegisterAgent,
            agent_id: self.id,
            payload: Some(json!({
                "capabilities": self.config.capabilities,
                "labels": self.config.labels,
            })),
        }).await?;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                // 接收命令
                Some(cmd) = self.recv_command() => {
                    self.handle_command(cmd).await?;
                }

                // 发送心跳
                _ = self.heartbeat_tick() => {
                    self.send_heartbeat().await?;
                }

                // 处理任务完成
                Some(result) = self.executor.next_result() => {
                    self.report_task_result(result).await?;
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: Command) -> Result<()> {
        match cmd.kind {
            CommandKind::TaskAcquired => {
                let task: TaskForExecute = serde_json::from_value(cmd.payload.unwrap())?;
                self.executor.submit(task).await?;
                self.send_ack().await?;
            }
            CommandKind::CancelTask => {
                let task_id: Uuid = serde_json::from_value(cmd.payload.unwrap())?;
                self.executor.cancel(task_id).await?;
                self.send_ack().await?;
            }
            CommandKind::Shutdown => {
                self.shutdown().await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

### TaskExecutor

```rust
use hetuflow_core::models::TaskConfig;
use hetuflow_core::types::ExecuteCommand;

pub struct TaskExecutor {
    tasks: DashMap<Uuid, JoinHandle<()>>,
    results: mpsc::Sender<TaskResult>,
}

impl TaskExecutor {
    pub async fn submit(&self, task: TaskForExecute) -> Result<()> {
        let config = task.config.clone();
        let results = self.results.clone();

        let handle = tokio::spawn(async move {
            let result = execute_task(&task.id, &config).await;
            let _ = results.send(result).await;
        });

        self.tasks.insert(task.id, handle);
        Ok(())
    }

    pub async fn cancel(&self, task_id: Uuid) -> Result<()> {
        if let Some((_, handle)) = self.tasks.remove(&task_id) {
            handle.abort();
        }
        Ok(())
    }
}

async fn execute_task(task_id: &Uuid, config: &TaskConfig) -> TaskResult {
    let start = std::time::Instant::now();

    let output = match config.cmd {
        ExecuteCommand::Bash => BashExecutor::execute(&config.args, config.timeout).await?,
        ExecuteCommand::Python => PythonExecutor::execute(&config.args, config.timeout).await?,
        ExecuteCommand::Node => NodeExecutor::execute(&config.args, config.timeout).await?,
        _ => return TaskResult::unsupported("Unsupported command type"),
    };

    TaskResult {
        task_id: *task_id,
        status: if output.status.success() {
            TaskInstanceStatus::Succeeded
        } else {
            TaskInstanceStatus::Failed
        },
        exit_code: Some(output.status.code()),
        stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
        stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        duration_ms: start.elapsed().as_millis() as u64,
    }
}
```

## Executors

### BashExecutor

```rust
pub struct BashExecutor;

impl BashExecutor {
    pub async fn execute(args: &[String], timeout: u32) -> Result<Output> {
        let cmd = tokio::process::Command::new("bash")
            .args(&["-c", &args.join(" ")])
            .output()
            .timeout(Duration::from_secs(timeout as u64))
            .await??;

        Ok(cmd)
    }
}
```

### PythonExecutor

```rust
pub struct PythonExecutor;

impl PythonExecutor {
    pub async fn execute(args: &[String], timeout: u32) -> Result<Output> {
        let cmd = tokio::process::Command::new("python")
            .args(args)
            .output()
            .timeout(Duration::from_secs(timeout as u64))
            .await??;

        Ok(cmd)
    }
}
```

## Heartbeat

```rust
impl Agent {
    pub async fn send_heartbeat(&self) -> Result<()> {
        self.send_event(Event {
            kind: EventKind::Heartbeat,
            agent_id: self.id,
            payload: Some(json!({
                "status": self.status,
                "running_tasks": self.executor.running_count(),
                "cpu_usage": get_cpu_usage(),
                "memory_usage": get_memory_usage(),
            })),
        }).await
    }
}
```

## WebSocket Connection

```rust
use tokio_tungstenite::WebSocketStream;
use futures::{SinkExt, StreamExt};

pub struct WebSocketConnection {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebSocketConnection {
    pub async fn send(&mut self, event: Event) -> Result<()> {
        let msg = serde_json::to_string(&event)?;
        self.ws.send(Message::Text(msg)).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Option<Command>> {
        match self.ws.next().await {
            Some(Ok(Message::Text(msg))) => {
                let cmd = serde_json::from_str(&msg)?;
                Ok(Some(cmd))
            }
            _ => Ok(None),
        }
    }
}
```

## Agent Config

```rust
pub struct AgentConfig {
    pub id: Uuid,
    pub server_url: String,
    pub labels: HashMap<String, String>,
    pub capabilities: Vec<String>,
    pub max_concurrent_tasks: usize,
    pub heartbeat_interval: Duration,
}

// TOML 配置
// [agent]
// id = "uuid"
// server_url = "ws://localhost:8080/ws"
// labels = { region = "us-east-1", env = "prod" }
// capabilities = ["bash", "python", "node"]
// max_concurrent_tasks = 10
// heartbeat_interval = "30s"
```

## Best Practices

1. **超时控制**: 所有任务执行都要设置超时
2. **资源限制**: 使用 cgroups 或容器限制资源使用
3. **日志采集**: 实时上报任务输出
4. **优雅关闭**: 收到 Shutdown 命令后完成当前任务再退出

## Examples from Codebase

- `hetuflow/hetuflow-agent/src/main.rs` - Agent 入口
- `hetuflow/hetuflow-agent/src/executor/mod.rs` - 执行器
