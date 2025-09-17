# Hetumind 本地运行器设计

## 1. 本地运行器概述

Hetumind 本地运行器是一个功能完整的 CLI 工具，提供本地工作流开发、测试和调试环境。它支持离线运行、实时调试和性能分析，是开发者的首选工具。

### 1.1 设计目标

- **开发友好**: 提供完整的本地开发环境
- **快速迭代**: 支持工作流的快速测试和调试
- **离线运行**: 无需网络连接即可运行工作流
- **性能分析**: 内置性能监控和分析工具
- **易于使用**: 直观的命令行界面

### 1.2 核心特性

- 工作流本地执行和调试
- 实时日志和状态监控
- 性能分析和优化建议
- 配置管理和环境变量
- 插件系统和扩展支持

## 2. CLI 架构设计

### 2.1 命令结构

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hetumind")]
#[command(about = "Hetumind 工作流自动化本地运行器")]
#[command(version = "1.0.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// 配置文件路径
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// 详细输出
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// 工作目录
    #[arg(short = 'C', long, global = true)]
    pub directory: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 工作流管理
    Workflow {
        #[command(subcommand)]
        action: WorkflowCommands,
    },
    /// 执行管理
    Execute {
        #[command(subcommand)]
        action: ExecuteCommands,
    },
    /// 节点管理
    Node {
        #[command(subcommand)]
        action: NodeCommands,
    },
    /// 开发工具
    Dev {
        #[command(subcommand)]
        action: DevCommands,
    },
    /// 配置管理
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}
```

### 2.2 工作流命令

```rust
#[derive(Subcommand)]
pub enum WorkflowCommands {
    /// 创建新工作流
    New {
        /// 工作流名称
        name: String,
        /// 模板类型
        #[arg(short, long)]
        template: Option<String>,
        /// 输出目录
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 列出工作流
    List {
        /// 过滤模式
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// 验证工作流
    Validate {
        /// 工作流文件路径
        file: PathBuf,
    },
    /// 运行工作流
    Run {
        /// 工作流文件路径
        file: PathBuf,
        /// 输入数据文件
        #[arg(short, long)]
        input: Option<PathBuf>,
        /// 环境变量文件
        #[arg(short, long)]
        env: Option<PathBuf>,
        /// 调试模式
        #[arg(short, long)]
        debug: bool,
        /// 监视模式
        #[arg(short, long)]
        watch: bool,
    },
    /// 导出工作流
    Export {
        /// 工作流文件路径
        file: PathBuf,
        /// 导出格式
        #[arg(short, long, default_value = "json")]
        format: ExportFormat,
        /// 输出文件
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    Json,
    Yaml,
    Toml,
}
```

### 2.3 执行命令

```rust
#[derive(Subcommand)]
pub enum ExecuteCommands {
    /// 运行单个节点
    Node {
        /// 节点类型
        node_type: String,
        /// 节点参数文件
        #[arg(short, long)]
        params: Option<PathBuf>,
        /// 输入数据文件
        #[arg(short, long)]
        input: Option<PathBuf>,
    },
    /// 交互式执行
    Interactive {
        /// 工作流文件路径
        file: PathBuf,
    },
    /// 批量执行
    Batch {
        /// 工作流文件路径
        file: PathBuf,
        /// 输入数据目录
        #[arg(short, long)]
        input_dir: PathBuf,
        /// 输出目录
        #[arg(short, long)]
        output_dir: PathBuf,
        /// 并发数
        #[arg(short, long, default_value = "4")]
        concurrency: usize,
    },
    /// 性能测试
    Benchmark {
        /// 工作流文件路径
        file: PathBuf,
        /// 测试次数
        #[arg(short, long, default_value = "10")]
        iterations: usize,
        /// 输出报告文件
        #[arg(short, long)]
        report: Option<PathBuf>,
    },
}
```

## 3. 本地执行引擎

### 3.1 本地执行器

```rust
use tokio::sync::mpsc;
use std::collections::HashMap;

pub struct LocalWorkflowRunner {
    /// 节点注册表
    node_registry: Arc<NodeRegistry>,
    /// 配置管理器
    config_manager: Arc<ConfigManager>,
    /// 事件发送器
    event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    /// 调试模式
    debug_mode: bool,
}

impl LocalWorkflowRunner {
    pub fn new(config: LocalRunnerConfig) -> Self {
        let (event_sender, _) = mpsc::unbounded_channel();

        Self {
            node_registry: Arc::new(NodeRegistry::new()),
            config_manager: Arc::new(ConfigManager::new(config.config_path)),
            event_sender,
            debug_mode: config.debug_mode,
        }
    }

    pub async fn run_workflow(
        &self,
        workflow_file: &Path,
        input_data: Option<Vec<ExecutionData>>,
        env_vars: HashMap<String, String>,
    ) -> Result<ExecutionResult, LocalRunnerError> {
        // 加载工作流定义
        let workflow = self.load_workflow(workflow_file).await?;

        // 验证工作流
        self.validate_workflow(&workflow)?;

        // 创建执行上下文
        let execution_context = ExecutionContext {
            execution_id: Uuid::now_v7(),
            workflow: workflow.clone(),
            current_node_id: Uuid::now_v7(),
            input_data: input_data.unwrap_or_default(),
            started_at: Utc::now(),
            mode: ExecutionMode::Local,
            user_id: None,
            env_vars,
        };

        // 创建本地执行引擎
        let engine = LocalExecutionEngine::new(
            self.node_registry.clone(),
            self.event_sender.clone(),
            self.debug_mode,
        );

        // 执行工作流
        engine.execute_workflow(&workflow, &execution_context).await
    }

    async fn load_workflow(&self, file_path: &Path) -> Result<Workflow, LocalRunnerError> {
        let content = tokio::fs::read_to_string(file_path).await?;

        match file_path.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| LocalRunnerError::ParseError(e.to_string())),
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
                .map_err(|e| LocalRunnerError::ParseError(e.to_string())),
            Some("toml") => toml::from_str(&content)
                .map_err(|e| LocalRunnerError::ParseError(e.to_string())),
            _ => Err(LocalRunnerError::UnsupportedFormat),
        }
    }

    fn validate_workflow(&self, workflow: &Workflow) -> Result<(), LocalRunnerError> {
        // 验证节点类型是否存在
        for node in &workflow.nodes {
            if !self.node_registry.has_node(&node.kind) {
                return Err(LocalRunnerError::UnknownNodeType(node.kind.clone()));
            }
        }

        // 验证连接的有效性
        for connection in &workflow.connections {
            let source_exists = workflow.nodes.iter()
                .any(|n| n.id == connection.source_node_id);
            let target_exists = workflow.nodes.iter()
                .any(|n| n.id == connection.target_node_id);

            if !source_exists || !target_exists {
                return Err(LocalRunnerError::InvalidConnection);
            }
        }

        Ok(())
    }
}
```

### 3.2 本地执行引擎

```rust
pub struct LocalExecutionEngine {
    node_registry: Arc<NodeRegistry>,
    event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    debug_mode: bool,
    execution_state: Arc<RwLock<HashMap<NodeId, NodeExecutionState>>>,
}

impl LocalExecutionEngine {
    pub async fn execute_workflow(
        &self,
        workflow: &Workflow,
        context: &ExecutionContext,
    ) -> Result<ExecutionResult, LocalRunnerError> {
        let start_time = Instant::now();

        // 发送执行开始事件
        self.send_event(ExecutionEvent {
            event_type: ExecutionEventType::ExecutionStarted,
            execution_id: context.execution_id,
            workflow_id: workflow.id,
            user_id: context.user_id.unwrap_or_default(),
            timestamp: Utc::now(),
            data: serde_json::json!({
                "workflow_name": workflow.name,
                "node_count": workflow.nodes.len(),
            }),
        });

        // 构建执行图
        let execution_graph = self.build_execution_graph(workflow)?;

        // 执行节点
        let mut execution_results = HashMap::default();
        let mut pending_nodes = execution_graph.get_start_nodes();

        while !pending_nodes.is_empty() {
            let mut next_nodes = Vec::new();

            // 并发执行当前层的节点
            let tasks: Vec<_> = pending_nodes.into_iter().map(|node_name| {
                let node = workflow.nodes.iter()
                    .find(|n| n.id == node_name)
                    .unwrap()
                    .clone();

                self.execute_node(node, context, &execution_results)
            }).collect();

            let results = futures::future::join_all(tasks).await;

            for (i, result) in results.into_iter().enumerate() {
                match result {
                    Ok(node_result) => {
                        execution_results.insert(node_result.node_name, node_result.clone());

                        // 获取下一层可执行的节点
                        let next = execution_graph.get_next_nodes(node_result.node_name);
                        next_nodes.extend(next);

                        if self.debug_mode {
                            println!("✓ 节点 {} 执行完成", node_result.node_name);
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ 节点执行失败: {}", e);
                        return Err(e);
                    }
                }
            }

            pending_nodes = next_nodes;
        }

        let duration = start_time.elapsed();

        // 发送执行完成事件
        self.send_event(ExecutionEvent {
            event_type: ExecutionEventType::ExecutionCompleted,
            execution_id: context.execution_id,
            workflow_id: workflow.id,
            user_id: context.user_id.unwrap_or_default(),
            timestamp: Utc::now(),
            data: serde_json::json!({
                "duration_ms": duration.as_millis(),
                "nodes_executed": execution_results.len(),
            }),
        });

        Ok(ExecutionResult {
            execution_id: context.execution_id,
            workflow_id: workflow.id,
            status: ExecutionStatus::Success,
            started_at: context.started_at,
            finished_at: Some(Utc::now()),
            duration: Some(duration),
            node_results: execution_results,
            error: None,
        })
    }

    async fn execute_node(
        &self,
        node: Node,
        context: &ExecutionContext,
        previous_results: &HashMap<NodeId, NodeExecutionResult>,
    ) -> Result<NodeExecutionResult, LocalRunnerError> {
        let start_time = Instant::now();

        // 发送节点开始事件
        self.send_event(ExecutionEvent {
            event_type: ExecutionEventType::NodeStarted,
            execution_id: context.execution_id,
            workflow_id: context.workflow.id,
            user_id: context.user_id.unwrap_or_default(),
            timestamp: Utc::now(),
            data: serde_json::json!({
                "node_name": node.id,
                "node_type": node.kind,
                "node_name": node.name,
            }),
        });

        // 获取节点执行器
        let executor = self.node_registry.get_executor(&node.kind)
            .ok_or_else(|| LocalRunnerError::UnknownNodeType(node.kind.clone()))?;

        // 准备输入数据
        let input_data = self.prepare_node_input(&node, previous_results, context)?;

        // 创建节点执行上下文
        let node_context = ExecutionContext {
            current_node_id: node.id,
            input_data,
            ..context.clone()
        };

        // 执行节点
        let result = match executor.execute(&node_context, &node).await {
            Ok(output_data) => {
                let duration = start_time.elapsed();

                if self.debug_mode {
                    println!("节点 {} ({}) 执行成功，耗时: {:?}",
                        node.name, node.kind, duration);
                    println!("输出数据: {}", serde_json::to_string_pretty(&output_data).unwrap());
                }

                NodeExecutionResult {
                    node_name: node.id,
                    status: ExecutionStatus::Success,
                    started_at: node_context.started_at,
                    finished_at: Some(Utc::now()),
                    duration: Some(duration),
                    input_data: node_context.input_data,
                    output_data,
                    error: None,
                }
            }
            Err(e) => {
                let duration = start_time.elapsed();

                if self.debug_mode {
                    eprintln!("节点 {} ({}) 执行失败: {}",
                        node.name, node.kind, e);
                }

                NodeExecutionResult {
                    node_name: node.id,
                    status: ExecutionStatus::Failed,
                    started_at: node_context.started_at,
                    finished_at: Some(Utc::now()),
                    duration: Some(duration),
                    input_data: node_context.input_data,
                    output_data: vec![],
                    error: Some(e.to_string()),
                }
            }
        };

        // 发送节点完成事件
        self.send_event(ExecutionEvent {
            event_type: if result.status == ExecutionStatus::Success {
                ExecutionEventType::NodeCompleted
            } else {
                ExecutionEventType::NodeFailed
            },
            execution_id: context.execution_id,
            workflow_id: context.workflow.id,
            user_id: context.user_id.unwrap_or_default(),
            timestamp: Utc::now(),
            data: serde_json::json!({
                "node_name": node.id,
                "duration_ms": result.duration.map(|d| d.as_millis()),
                "error": result.error,
            }),
        });

        Ok(result)
    }
}
```

## 4. 开发工具

### 4.1 交互式调试器

```rust
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

pub struct InteractiveDebugger {
    workflow: Workflow,
    execution_state: ExecutionState,
    current_node: Option<NodeId>,
    breakpoints: HashSet<NodeId>,
}

impl InteractiveDebugger {
    pub async fn start_debug_session(
        &mut self,
        workflow_file: &Path,
    ) -> Result<(), LocalRunnerError> {
        // 初始化终端
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // 加载工作流
        self.workflow = self.load_workflow(workflow_file).await?;

        // 主调试循环
        loop {
            // 绘制界面
            terminal.draw(|f| self.draw_debug_ui(f))?;

            // 处理用户输入
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('n') => self.step_next().await?,
                    KeyCode::Char('c') => self.continue_execution().await?,
                    KeyCode::Char('b') => self.toggle_breakpoint(),
                    KeyCode::Char('r') => self.restart_execution().await?,
                    _ => {}
                }
            }
        }

        // 清理终端
        disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;

        Ok(())
    }

    fn draw_debug_ui(&self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(f.size());

        // 工作流概览
        let workflow_block = Block::default()
            .title("工作流概览")
            .borders(Borders::ALL);
        let workflow_info = Paragraph::new(format!(
            "名称: {}\n节点数: {}\n连接数: {}",
            self.workflow.name,
            self.workflow.nodes.len(),
            self.workflow.connections.len()
        ))
        .block(workflow_block);
        f.render_widget(workflow_info, chunks[0]);

        // 节点列表
        let nodes_block = Block::default()
            .title("节点列表")
            .borders(Borders::ALL);
        let node_items: Vec<ListItem> = self.workflow.nodes.iter()
            .map(|node| {
                let status = if Some(node.id) == self.current_node {
                    "▶ "
                } else if self.breakpoints.contains(&node.id) {
                    "● "
                } else {
                    "  "
                };
                ListItem::new(format!("{}{} ({})", status, node.name, node.kind))
            })
            .collect();
        let nodes_list = List::new(node_items).block(nodes_block);
        f.render_widget(nodes_list, chunks[1]);

        // 控制面板
        let controls_block = Block::default()
            .title("控制面板")
            .borders(Borders::ALL);
        let controls_text = "快捷键:\n\
            n - 单步执行\n\
            c - 继续执行\n\
            b - 切换断点\n\
            r - 重新开始\n\
            q - 退出";
        let controls = Paragraph::new(controls_text).block(controls_block);
        f.render_widget(controls, chunks[2]);
    }
}
```

### 4.2 性能分析器

```rust
pub struct PerformanceProfiler {
    execution_metrics: HashMap<NodeId, NodeMetrics>,
    workflow_metrics: WorkflowMetrics,
    start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct NodeMetrics {
    pub execution_count: u64,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub memory_usage: u64,
    pub cpu_usage: f64,
}

#[derive(Debug, Clone)]
pub struct WorkflowMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub avg_execution_time: Duration,
    pub throughput: f64, // executions per second
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            execution_metrics: HashMap::default(),
            workflow_metrics: WorkflowMetrics::default(),
            start_time: Instant::now(),
        }
    }

    pub fn record_node_execution(
        &mut self,
        node_name: NodeId,
        duration: Duration,
        memory_usage: u64,
        cpu_usage: f64,
    ) {
        let metrics = self.execution_metrics.entry(node_name).or_insert_with(|| {
            NodeMetrics {
                execution_count: 0,
                total_duration: Duration::ZERO,
                avg_duration: Duration::ZERO,
                min_duration: Duration::MAX,
                max_duration: Duration::ZERO,
                memory_usage: 0,
                cpu_usage: 0.0,
            }
        });

        metrics.execution_count += 1;
        metrics.total_duration += duration;
        metrics.avg_duration = metrics.total_duration / metrics.execution_count as u32;
        metrics.min_duration = metrics.min_duration.min(duration);
        metrics.max_duration = metrics.max_duration.max(duration);
        metrics.memory_usage = metrics.memory_usage.max(memory_usage);
        metrics.cpu_usage = (metrics.cpu_usage + cpu_usage) / 2.0;
    }

    pub fn generate_report(&self) -> PerformanceReport {
        let total_time = self.start_time.elapsed();

        PerformanceReport {
            workflow_metrics: self.workflow_metrics.clone(),
            node_metrics: self.execution_metrics.clone(),
            total_time,
            recommendations: self.generate_recommendations(),
        }
    }

    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // 分析慢节点
        for (node_name, metrics) in &self.execution_metrics {
            if metrics.avg_duration > Duration::from_secs(5) {
                recommendations.push(format!(
                    "节点 {} 平均执行时间较长 ({:?})，建议优化",
                    node_name, metrics.avg_duration
                ));
            }

            if metrics.memory_usage > 1024 * 1024 * 100 { // 100MB
                recommendations.push(format!(
                    "节点 {} 内存使用量较高 ({} MB)，建议检查内存泄漏",
                    node_name, metrics.memory_usage / 1024 / 1024
                ));
            }
        }

        recommendations
    }
}
```

## 5. 配置管理

### 5.1 配置系统

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRunnerConfig {
    /// 工作目录
    pub workspace: PathBuf,
    /// 日志级别
    pub log_level: String,
    /// 调试模式
    pub debug_mode: bool,
    /// 并发限制
    pub max_concurrency: usize,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// 节点配置
    pub node_configs: HashMap<String, serde_json::Value>,
    /// 插件目录
    pub plugin_dirs: Vec<PathBuf>,
}

impl Default for LocalRunnerConfig {
    fn default() -> Self {
        Self {
            workspace: PathBuf::from("."),
            log_level: "info".to_string(),
            debug_mode: false,
            max_concurrency: num_cpus::get(),
            env_vars: HashMap::default(),
            node_configs: HashMap::default(),
            plugin_dirs: vec![PathBuf::from("./plugins")],
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: LocalRunnerConfig,
}

impl ConfigManager {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        let config_path = config_path.unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("hetumind")
                .join("config.toml")
        });

        let config = Self::load_config(&config_path)
            .unwrap_or_default();

        Self { config_path, config }
    }

    fn load_config(path: &Path) -> Result<LocalRunnerConfig, ConfigError> {
        if !path.exists() {
            return Ok(LocalRunnerConfig::default());
        }

        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(ConfigError::ParseError)
    }

    pub fn save_config(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, content)?;

        Ok(())
    }

    pub fn get_config(&self) -> &LocalRunnerConfig {
        &self.config
    }

    pub fn update_config<F>(&mut self, updater: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut LocalRunnerConfig),
    {
        updater(&mut self.config);
        self.save_config()
    }
}
```

## 6. 监控和日志

### 6.1 实时监控

```rust
use tokio::sync::broadcast;

pub struct LocalMonitor {
    event_receiver: broadcast::Receiver<ExecutionEvent>,
    metrics_collector: Arc<Mutex<MetricsCollector>>,
}

impl LocalMonitor {
    pub async fn start_monitoring(&mut self) {
        println!("🚀 开始监控工作流执行...");

        while let Ok(event) = self.event_receiver.recv().await {
            self.handle_event(event).await;
        }
    }

    async fn handle_event(&self, event: ExecutionEvent) {
        match event.event_type {
            ExecutionEventType::ExecutionStarted => {
                println!("▶️  执行开始: {}", event.execution_id);
            }
            ExecutionEventType::NodeStarted => {
                if let Some(node_name) = event.data.get("node_name") {
                    println!("  🔄 节点开始: {}", node_name);
                }
            }
            ExecutionEventType::NodeCompleted => {
                if let Some(duration) = event.data.get("duration_ms") {
                    println!("  ✅ 节点完成，耗时: {}ms", duration);
                }
            }
            ExecutionEventType::NodeFailed => {
                if let Some(error) = event.data.get("error") {
                    println!("  ❌ 节点失败: {}", error);
                }
            }
            ExecutionEventType::ExecutionCompleted => {
                if let Some(duration) = event.data.get("duration_ms") {
                    println!("🎉 执行完成，总耗时: {}ms", duration);
                }
            }
            ExecutionEventType::ExecutionFailed => {
                println!("💥 执行失败");
            }
            _ => {}
        }

        // 收集指标
        let mut collector = self.metrics_collector.lock().await;
        collector.record_event(&event);
    }
}
```

这个本地运行器设计为 Hetumind 系统提供了完整的本地开发和调试环境，支持交互式调试、性能分析和实时监控，是开发者进行工作流开发的理想工具。
