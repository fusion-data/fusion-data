//! Graph-flow based Memory Node Implementation
//!
//! 基于 graph-flow 框架重构的内存节点，使用 InMemorySessionStorage 和 InMemoryGraphStorage
//! 提供工作流级别的内存管理和会话隔离功能

use async_trait::async_trait;
use fusion_ai::graph_flow::{
  Context, FlowRunner, GraphBuilder, GraphStorage, InMemoryGraphStorage, InMemorySessionStorage, NextAction, Session,
  SessionStorage, Task, TaskResult,
};
use fusion_common::time::now_offset;
use hetumind_core::types::JsonValue;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::VecDeque;
use std::sync::Arc;
use uuid::Uuid;

/// 图流内存任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphFlowMemoryConfig {
  /// 会话ID
  pub session_id: String,
  /// 上下文窗口长度
  pub context_window_length: usize,
  /// 是否启用持久化
  pub persistence_enabled: bool,
  /// 输入键名
  pub input_key: String,
  /// 内存键名
  pub memory_key: String,
  /// 输出键名
  pub output_key: String,
}

impl Default for GraphFlowMemoryConfig {
  fn default() -> Self {
    Self {
      session_id: Uuid::new_v4().to_string(),
      context_window_length: 5,
      persistence_enabled: false,
      input_key: "input".to_string(),
      memory_key: "chat_history".to_string(),
      output_key: "output".to_string(),
    }
  }
}

/// 会话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphFlowConversationMessage {
  /// 消息ID
  pub message_id: String,
  /// 消息角色
  pub role: String,
  /// 消息内容
  pub content: String,
  /// 时间戳
  pub timestamp: chrono::DateTime<chrono::FixedOffset>,
  /// 元数据
  pub metadata: Option<JsonValue>,
}

impl GraphFlowConversationMessage {
  pub fn new(role: String, content: String) -> Self {
    Self { message_id: Uuid::new_v4().to_string(), role, content, timestamp: now_offset(), metadata: None }
  }

  pub fn with_metadata(role: String, content: String, metadata: JsonValue) -> Self {
    Self { message_id: Uuid::new_v4().to_string(), role, content, timestamp: now_offset(), metadata: Some(metadata) }
  }
}

/// 内存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphFlowMemoryStats {
  /// 总消息数量
  pub total_messages: usize,
  /// 会话ID
  pub session_id: String,
  /// 上下文窗口长度
  pub context_window_length: usize,
  /// 统计时间戳
  pub timestamp: chrono::DateTime<chrono::FixedOffset>,
}

impl GraphFlowMemoryStats {
  pub fn new(total_messages: usize, session_id: String, context_window_length: usize) -> Self {
    Self { total_messages, session_id, context_window_length, timestamp: now_offset() }
  }
}

/// 内存数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphFlowMemoryData {
  /// 会话ID
  pub session_id: String,
  /// 工作流ID
  pub workflow_id: String,
  /// 配置
  pub config: GraphFlowMemoryConfig,
  /// 聊天历史
  pub chat_history: VecDeque<GraphFlowConversationMessage>,
  /// 统计信息
  pub stats: GraphFlowMemoryStats,
}

impl GraphFlowMemoryData {
  pub fn new(session_id: String, workflow_id: String, config: GraphFlowMemoryConfig) -> Self {
    let stats = GraphFlowMemoryStats::new(0, session_id.clone(), config.context_window_length);
    Self { session_id, workflow_id, config, chat_history: VecDeque::new(), stats }
  }

  /// 添加消息到内存
  pub fn add_message(&mut self, message: GraphFlowConversationMessage) {
    self.chat_history.push_back(message.clone());

    // 滑动窗口管理
    while self.chat_history.len() > self.config.context_window_length {
      self.chat_history.pop_front();
    }

    // 更新统计信息
    self.stats.total_messages = self.chat_history.len();
    self.stats.timestamp = now_offset();

    info!("Added message to memory buffer. Current size: {}", self.chat_history.len());
  }

  /// 获取最近的N条消息
  pub fn get_recent_messages(&self, count: usize) -> Vec<GraphFlowConversationMessage> {
    let buffer_len = self.chat_history.len();
    if buffer_len <= count {
      self.chat_history.iter().cloned().collect()
    } else {
      self.chat_history.range(buffer_len - count..).cloned().collect()
    }
  }

  /// 获取所有消息
  pub fn get_all_messages(&self) -> Vec<GraphFlowConversationMessage> {
    self.chat_history.iter().cloned().collect()
  }

  /// 获取消息数量
  pub fn len(&self) -> usize {
    self.chat_history.len()
  }

  /// 检查是否为空
  pub fn is_empty(&self) -> bool {
    self.chat_history.is_empty()
  }

  /// 清空内存
  pub fn clear(&mut self) {
    self.chat_history.clear();
    self.stats.total_messages = 0;
    self.stats.timestamp = now_offset();
  }
}

/// 图流内存任务 - 存储消息
pub struct GraphFlowMemoryStoreTask;

#[async_trait]
impl Task for GraphFlowMemoryStoreTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowMemoryStoreTask");

    // 获取输入消息
    let input_data: JsonValue = context.get_sync("input_message").unwrap_or_else(|| json!({}));

    debug!("Input message data: {}", serde_json::to_string_pretty(&input_data).unwrap_or_default());

    // 获取或创建内存数据
    let mut memory_data: GraphFlowMemoryData = context.get_sync("memory_data").unwrap_or_else(|| {
      let session_id: String = context.get_sync("session_id").unwrap_or_else(|| "default_session".to_string());
      let workflow_id: String = context.get_sync("workflow_id").unwrap_or_else(|| "default_workflow".to_string());
      let config: GraphFlowMemoryConfig = context.get_sync("memory_config").unwrap_or_default();
      GraphFlowMemoryData::new(session_id, workflow_id, config)
    });

    // 解析输入消息
    if let Some(messages) = input_data.get("messages").and_then(|v| v.as_array()) {
      // 处理消息数组
      for msg in messages {
        if let (Some(role), Some(content)) =
          (msg.get("role").and_then(|v| v.as_str()), msg.get("content").and_then(|v| v.as_str()))
        {
          let mut graph_msg = GraphFlowConversationMessage::new(role.to_string(), content.to_string());

          // 提取可选的元数据
          if let Some(metadata) = msg.get("metadata") {
            graph_msg.metadata = Some(metadata.clone());
          }

          memory_data.add_message(graph_msg);
        }
      }
    } else if let (Some(role), Some(content)) =
      (input_data.get("role").and_then(|v| v.as_str()), input_data.get("content").and_then(|v| v.as_str()))
    {
      // 处理单条消息
      let mut graph_msg = GraphFlowConversationMessage::new(role.to_string(), content.to_string());

      if let Some(metadata) = input_data.get("metadata") {
        graph_msg.metadata = Some(metadata.clone());
      }

      memory_data.add_message(graph_msg);
    }

    // 保存更新后的内存数据到上下文
    context.set("memory_data", memory_data.clone()).await;

    // 返回结果
    let result = json!({
      "session_id": memory_data.session_id,
      "workflow_id": memory_data.workflow_id,
      "messages_stored": memory_data.len(),
      "stats": memory_data.stats,
      "timestamp": now_offset(),
    });

    Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
  }
}

/// 图流内存任务 - 检索消息
pub struct GraphFlowMemoryRetrieveTask;

#[async_trait]
impl Task for GraphFlowMemoryRetrieveTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowMemoryRetrieveTask");

    // 获取检索参数
    let count: usize = context.get_sync("retrieve_count").unwrap_or(5);

    // 获取内存数据
    let memory_data: Option<GraphFlowMemoryData> = context.get_sync("memory_data");

    match memory_data {
      Some(data) => {
        let recent_messages = data.get_recent_messages(count);

        debug!("Retrieved {} messages from memory", recent_messages.len());

        let result = json!({
          "session_id": data.session_id,
          "workflow_id": data.workflow_id,
          "chat_history": recent_messages,
          "stats": data.stats,
          "total_messages": data.len(),
          "timestamp": now_offset(),
        });

        Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
      }
      None => {
        warn!("No memory data found for retrieval");
        let result = json!({
          "session_id": context.get_sync::<String>("session_id").unwrap_or_else(|| "unknown".to_string()),
          "chat_history": [],
          "stats": null,
          "total_messages": 0,
          "message": "No memory data found",
          "timestamp": now_offset(),
        });

        Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
      }
    }
  }
}

/// 图流内存管理器
#[derive(Clone)]
pub struct GraphFlowMemoryManager {
  /// 会话存储
  session_storage: Arc<dyn SessionStorage>,
  /// 图存储
  graph_storage: Arc<dyn GraphStorage>,
  /// 工作流运行器
  runner: FlowRunner,
}

impl GraphFlowMemoryManager {
  /// 创建新的内存管理器
  pub fn new() -> Self {
    let session_storage: Arc<dyn SessionStorage> = Arc::new(InMemorySessionStorage::new());
    let graph_storage: Arc<dyn GraphStorage> = Arc::new(InMemoryGraphStorage::new());

    // 创建一个简单的空图用于内存管理
    let graph = Arc::new(GraphBuilder::new("memory_graph").build());

    Self {
      session_storage: session_storage.clone(),
      graph_storage: graph_storage.clone(),
      runner: FlowRunner::new(graph, session_storage),
    }
  }

  /// 存储消息到内存
  pub async fn store_messages(
    &self,
    session_id: &str,
    workflow_id: &str,
    messages: Vec<JsonValue>,
    config: Option<GraphFlowMemoryConfig>,
  ) -> Result<GraphFlowMemoryData, Box<dyn std::error::Error + Send + Sync>> {
    let config = config.unwrap_or_default();

    // 创建或获取会话
    let session_id_str = format!("memory_session_{}", session_id);
    let session = Session::new_from_task(session_id_str.clone(), "store_task");

    // 设置会话上下文
    session.context.set("session_id", session_id.to_string()).await;
    session.context.set("workflow_id", workflow_id.to_string()).await;
    session.context.set("memory_config", config.clone()).await;
    session.context.set("input_message", json!({ "messages": messages })).await;

    // 保存会话
    self.session_storage.save(session.clone()).await?;

    // 执行存储任务
    let execution_result = self.runner.run(&session_id_str).await?;

    if let Some(result) = execution_result.response {
      let memory_data: GraphFlowMemoryData = serde_json::from_str(&result.to_string())?;
      Ok(memory_data)
    } else {
      Err("No result from memory store task".into())
    }
  }

  /// 从内存检索消息
  pub async fn retrieve_messages(
    &self,
    session_id: &str,
    count: usize,
  ) -> Result<Vec<GraphFlowConversationMessage>, Box<dyn std::error::Error + Send + Sync>> {
    let session_id_str = format!("memory_session_{}", session_id);
    let session = Session::new_from_task(session_id_str.clone(), "retrieve_task");

    // 设置会话上下文
    session.context.set("session_id", session_id.to_string()).await;
    session.context.set("retrieve_count", count).await;

    // 保存会话
    self.session_storage.save(session.clone()).await?;

    // 执行检索任务
    let execution_result = self.runner.run(&session_id_str).await?;

    if let Some(result) = execution_result.response {
      let result_json: JsonValue = serde_json::from_str(&result.to_string())?;

      if let Some(chat_history) = result_json.get("chat_history").and_then(|v| v.as_array()) {
        let messages: Result<Vec<GraphFlowConversationMessage>, _> =
          chat_history.iter().map(|msg| serde_json::from_value(msg.clone())).collect();

        Ok(messages.unwrap_or_default())
      } else {
        Ok(vec![])
      }
    } else {
      Err("No result from memory retrieve task".into())
    }
  }

  /// 获取内存统计信息
  pub async fn get_memory_stats(
    &self,
    session_id: &str,
  ) -> Result<Option<GraphFlowMemoryStats>, Box<dyn std::error::Error + Send + Sync>> {
    let session_id_str = format!("memory_session_{}", session_id);
    let session = Session::new_from_task(session_id_str.clone(), "retrieve_task");

    // 设置会话上下文
    session.context.set("session_id", session_id.to_string()).await;
    session.context.set("retrieve_count", 0).await; // 只获取统计信息

    // 保存会话
    self.session_storage.save(session.clone()).await?;

    // 执行检索任务
    let execution_result = self.runner.run(&session_id_str).await?;

    if let Some(result) = execution_result.response {
      let result_json: JsonValue = serde_json::from_str(&result.to_string())?;

      if let Some(stats) = result_json.get("stats") {
        let stats: GraphFlowMemoryStats = serde_json::from_value(stats.clone())?;
        Ok(Some(stats))
      } else {
        Ok(None)
      }
    } else {
      Ok(None)
    }
  }

  /// 清空内存
  pub async fn clear_memory(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Clearing memory for session: {}", session_id);

    // 通过设置空消息来清空内存
    self.store_messages(session_id, "clear_workflow", vec![], None).await?;

    Ok(())
  }

  /// 列出所有活跃会话
  pub async fn list_active_sessions(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    // 这个功能需要 SessionStorage 提供列举功能，目前返回空列表
    // 在实际实现中可能需要扩展 InMemorySessionStorage 的接口
    warn!("Session listing not implemented in current InMemorySessionStorage");
    Ok(vec![])
  }
}

impl Default for GraphFlowMemoryManager {
  fn default() -> Self {
    Self::new()
  }
}
