//! Graph-flow based AI Agent Implementation
//!
//! 基于 graph-flow 框架重构的 AI Agent，将复杂的 AI 工作流分解为一系列任务
//! 支持 LLM 调用、内存管理和工具集成的统一协调

use async_trait::async_trait;
use fusion_ai::graph_flow::{
  Context, ExecutionStatus, FlowRunner, GraphBuilder, GraphStorage, InMemoryGraphStorage, InMemorySessionStorage,
  NextAction, Session, SessionStorage, Task, TaskResult,
};
use fusion_common::time::now_offset;
use hetumind_core::types::JsonValue;
use log::{debug, info, warn};
use rig::client::CompletionClient;
use rig::completion::Completion;
use rig::message::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::cluster::ai_agent::parameters::AiAgentConfig;
use crate::llm::UsageStats;
use crate::memory::graph_flow_memory::{GraphFlowConversationMessage, GraphFlowMemoryConfig, GraphFlowMemoryManager};

/// 图流 AI Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphFlowAgentConfig {
  /// 基础 AI Agent 配置
  pub base_config: AiAgentConfig,
  /// 会话ID
  pub session_id: String,
  /// 内存管理器配置
  pub memory_config: Option<GraphFlowMemoryConfig>,
  /// LLM 配置
  pub llm_config: Option<JsonValue>,
  /// 工具配置
  pub tools_config: Option<Vec<JsonValue>>,
}

impl Default for GraphFlowAgentConfig {
  fn default() -> Self {
    Self {
      base_config: AiAgentConfig::default(),
      session_id: Uuid::new_v4().to_string(),
      memory_config: None,
      llm_config: None,
      tools_config: None,
    }
  }
}

/// LLM 调用任务
pub struct GraphFlowLLMTask {
  /// API 密钥
  api_key: String,
  /// 模型名称
  model_name: String,
}

impl GraphFlowLLMTask {
  pub fn new(api_key: String, model_name: String) -> Self {
    Self { api_key, model_name }
  }
}

#[async_trait]
impl Task for GraphFlowLLMTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowLLMTask with model: {}", self.model_name);

    // 获取输入数据
    let input_data: JsonValue = context.get_sync("input_messages").unwrap_or_else(|| json!({}));

    debug!("Input data for LLM: {}", serde_json::to_string_pretty(&input_data).unwrap_or_default());

    // 创建 DeepSeek 客户端
    let client = rig::providers::deepseek::Client::new(&self.api_key);
    let mut agent_builder = client.agent(&self.model_name);

    // 设置系统提示词
    if let Some(system_prompt) = context.get_sync::<String>("system_prompt") {
      agent_builder = agent_builder.preamble(&system_prompt);
    }

    let agent = agent_builder.build();

    // 获取提示词
    let prompt_content = input_data.get("prompt").and_then(|v| v.as_str()).unwrap_or("");

    let prompt_message = Message::user(prompt_content.to_string());

    // 获取历史消息
    let mut history_messages = Vec::new();
    if let Some(history) = input_data.get("chat_history").and_then(|v| v.as_array()) {
      for msg in history {
        if let (Some(role), Some(content)) =
          (msg.get("role").and_then(|v| v.as_str()), msg.get("content").and_then(|v| v.as_str()))
        {
          match role {
            "user" => history_messages.push(Message::user(content.to_string())),
            "assistant" => history_messages.push(Message::assistant(content.to_string())),
            _ => {
              debug!("Unknown message role: {}, treating as user", role);
              history_messages.push(Message::user(content.to_string()));
            }
          }
        }
      }
    }

    debug!("Processing {} messages with LLM", history_messages.len() + 1);

    // 执行 LLM 调用
    let completion = agent.completion(prompt_message, history_messages).await;
    match completion {
      Ok(completion) => {
        info!("LLM completion successful");

        // 发送 completion 请求获取最终响应
        match completion.send().await {
          Ok(completion_response) => {
            // 提取响应文本
            let choice = completion_response.choice.first();
            let response_text = match choice {
              rig::completion::AssistantContent::Text(text) => text.text.clone(),
              _ => "".to_string(),
            };

            let result = json!({
              "content": response_text,
              "role": "assistant",
              "model": self.model_name,
              "timestamp": now_offset(),
              "usage": UsageStats::from(completion_response.usage)
            });

            Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
          }
          Err(e) => {
            warn!("LLM completion send failed: {}", e);
            let error_result = json!({
              "error": format!("Completion send failed: {}", e),
              "content": "",
              "role": "assistant",
              "model": self.model_name,
              "timestamp": now_offset(),
            });

            Ok(TaskResult::new(Some(error_result.to_string()), NextAction::Continue))
          }
        }
      }
      Err(e) => {
        warn!("LLM call failed: {}", e);
        let error_result = json!({
          "error": format!("LLM call failed: {}", e),
          "content": "",
          "role": "assistant",
          "model": self.model_name,
          "timestamp": now_offset(),
        });

        Ok(TaskResult::new(Some(error_result.to_string()), NextAction::Continue))
      }
    }
  }
}

/// 内存管理任务 - 存储消息
pub struct GraphFlowMemoryStoreTask {
  memory_manager: Arc<GraphFlowMemoryManager>,
}

impl GraphFlowMemoryStoreTask {
  pub fn new(memory_manager: Arc<GraphFlowMemoryManager>) -> Self {
    Self { memory_manager }
  }
}

#[async_trait]
impl Task for GraphFlowMemoryStoreTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowMemoryStoreTask");

    // 获取会话ID和工作流ID
    let session_id: String = context.get_sync("session_id").unwrap_or_else(|| "default_session".to_string());
    let workflow_id: String = context.get_sync("workflow_id").unwrap_or_else(|| "default_workflow".to_string());

    // 获取要存储的消息
    let messages: Vec<JsonValue> = context.get_sync("messages_to_store").unwrap_or_default();

    if messages.is_empty() {
      debug!("No messages to store");
      return Ok(TaskResult::new(Some(json!({"stored": 0}).to_string()), NextAction::Continue));
    }

    // 获取内存配置
    let memory_config: Option<GraphFlowMemoryConfig> = context.get_sync("memory_config");

    // 存储消息
    match self.memory_manager.store_messages(&session_id, &workflow_id, messages, memory_config).await {
      Ok(memory_data) => {
        info!("Successfully stored {} messages", memory_data.len());

        let result = json!({
          "session_id": memory_data.session_id,
          "stored_count": memory_data.len(),
          "stats": memory_data.stats,
          "timestamp": now_offset(),
        });

        Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
      }
      Err(e) => {
        warn!("Failed to store messages: {}", e);
        let error_result = json!({
          "error": format!("Failed to store messages: {}", e),
          "stored_count": 0,
          "timestamp": now_offset(),
        });

        Ok(TaskResult::new(Some(error_result.to_string()), NextAction::Continue))
      }
    }
  }
}

/// 内存管理任务 - 检索消息
pub struct GraphFlowMemoryRetrieveTask {
  memory_manager: Arc<GraphFlowMemoryManager>,
}

impl GraphFlowMemoryRetrieveTask {
  pub fn new(memory_manager: Arc<GraphFlowMemoryManager>) -> Self {
    Self { memory_manager }
  }
}

#[async_trait]
impl Task for GraphFlowMemoryRetrieveTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowMemoryRetrieveTask");

    // 获取会话ID和检索数量
    let session_id: String = context.get_sync("session_id").unwrap_or_else(|| "default_session".to_string());
    let retrieve_count: usize = context.get_sync("retrieve_count").unwrap_or(5);

    // 检索消息
    match self.memory_manager.retrieve_messages(&session_id, retrieve_count).await {
      Ok(messages) => {
        info!("Retrieved {} messages from memory", messages.len());

        let result = json!({
          "session_id": session_id,
          "chat_history": messages,
          "count": messages.len(),
          "timestamp": now_offset(),
        });

        Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
      }
      Err(e) => {
        warn!("Failed to retrieve messages: {}", e);
        let error_result = json!({
          "error": format!("Failed to retrieve messages: {}", e),
          "session_id": session_id,
          "chat_history": [],
          "count": 0,
          "timestamp": now_offset(),
        });

        Ok(TaskResult::new(Some(error_result.to_string()), NextAction::Continue))
      }
    }
  }
}

/// 消息准备任务
pub struct GraphFlowMessagePreparationTask;

#[async_trait]
impl Task for GraphFlowMessagePreparationTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowMessagePreparationTask");

    // 获取配置
    let config: GraphFlowAgentConfig = context.get_sync("agent_config").unwrap_or_default();

    // 获取当前用户输入
    let current_input: JsonValue = context.get_sync("user_input").unwrap_or_else(|| json!({}));

    // 获取历史消息
    let memory_history: Option<JsonValue> = context.get_sync("memory_history");

    let mut messages = Vec::new();

    // 添加历史对话
    if let Some(history) = memory_history
      && let Some(chat_history) = history.get("chat_history").and_then(|v| v.as_array())
    {
      for msg in chat_history {
        if let (Some(role), Some(content)) =
          (msg.get("role").and_then(|v| v.as_str()), msg.get("content").and_then(|v| v.as_str()))
        {
          match role {
            "user" => messages.push(Message::user(content.to_string())),
            "assistant" => messages.push(Message::assistant(content.to_string())),
            _ => {
              debug!("Unknown message role: {}", role);
              messages.push(Message::user(content.to_string()));
            }
          }
        }
      }
    }

    // 添加当前用户输入
    if let Some(content) = current_input.get("content").and_then(|v| v.as_str()) {
      messages.push(Message::user(content.to_string()));
    } else if let Some(prompt) = current_input.get("prompt").and_then(|v| v.as_str()) {
      messages.push(Message::user(prompt.to_string()));
    }

    debug!("Prepared {} messages for LLM", messages.len());

    // 准备要存储到内存的消息
    let mut messages_to_store = Vec::new();

    // 添加当前用户输入到存储列表
    if let Some(content) = current_input.get("content").and_then(|v| v.as_str()) {
      let user_msg = json!({
        "role": "user",
        "content": content,
        "timestamp": now_offset(),
      });
      messages_to_store.push(user_msg);
    }

    let result = json!({
      "messages": messages,
      "messages_to_store": messages_to_store,
      "current_input": current_input,
      "system_prompt": config.base_config.system_prompt(),
      "timestamp": now_offset(),
    });

    Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
  }
}

/// 响应后处理任务
pub struct GraphFlowResponsePostProcessTask;

#[async_trait]
impl Task for GraphFlowResponsePostProcessTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowResponsePostProcessTask");

    // 获取配置
    let config: GraphFlowAgentConfig = context.get_sync("agent_config").unwrap_or_default();

    // 获取 LLM 响应
    let llm_response: JsonValue = context.get_sync("llm_response").unwrap_or_else(|| json!({}));

    // 提取响应内容
    let response_content = llm_response.get("content").and_then(|v| v.as_str()).unwrap_or("");

    // 准备助手响应消息用于存储
    let assistant_msg = json!({
      "role": "assistant",
      "content": response_content,
      "timestamp": now_offset(),
      "model": llm_response.get("model").unwrap_or(&json!("unknown")),
    });

    // 构建最终响应
    let final_response = json!({
      "response": response_content,
      "role": "assistant",
      "model": llm_response.get("model").unwrap_or(&json!("unknown")),
      "session_id": config.session_id,
      "timestamp": now_offset(),
      "usage": llm_response.get("usage").unwrap_or(&json!({})),
      "memory_stats": {
        "session_id": config.session_id,
        "has_memory": config.memory_config.is_some(),
      },
      "streaming": config.base_config.enable_streaming(),
    });

    let result = json!({
      "final_response": final_response,
      "assistant_message": assistant_msg,
      "llm_response": llm_response,
      "timestamp": now_offset(),
    });

    Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
  }
}

/// 图流 AI Agent 工作流管理器
#[derive(Clone)]
pub struct GraphFlowAgentManager {
  /// 内存管理器
  memory_manager: Arc<GraphFlowMemoryManager>,
  /// 会话存储
  session_storage: Arc<dyn SessionStorage>,
  /// 图存储
  graph_storage: Arc<dyn GraphStorage>,
  /// 工作流运行器
  runner: FlowRunner,
}

impl GraphFlowAgentManager {
  /// 创建新的 AI Agent 管理器
  pub fn new() -> Self {
    let session_storage: Arc<dyn SessionStorage> = Arc::new(InMemorySessionStorage::new());
    let graph_storage: Arc<dyn GraphStorage> = Arc::new(InMemoryGraphStorage::new());
    let memory_manager = Arc::new(GraphFlowMemoryManager::new());

    // 创建一个简单的空图用于 AI Agent 管理
    let graph = Arc::new(GraphBuilder::new("agent_graph").build());

    Self {
      memory_manager,
      session_storage: session_storage.clone(),
      graph_storage,
      runner: FlowRunner::new(graph, session_storage),
    }
  }

  /// 执行完整的 AI Agent 工作流
  pub async fn execute_agent_workflow(
    &self,
    config: GraphFlowAgentConfig,
    user_input: JsonValue,
    api_key: String,
    model_name: String,
  ) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting AI Agent workflow for session: {}", config.session_id);

    // 创建工作流图
    let graph = Arc::new(
      GraphBuilder::new("ai_agent_workflow")
        // 任务定义
        .add_task(Arc::new(GraphFlowMessagePreparationTask))
        .add_task(Arc::new(GraphFlowMemoryRetrieveTask::new(self.memory_manager.clone())))
        .add_task(Arc::new(GraphFlowLLMTask::new(api_key, model_name)))
        .add_task(Arc::new(GraphFlowMemoryStoreTask::new(self.memory_manager.clone())))
        .add_task(Arc::new(GraphFlowResponsePostProcessTask))
        // 工作流连接
        .add_edge("graph_flow_message_preparation_task", "graph_flow_memory_retrieve_task")
        .add_edge("graph_flow_memory_retrieve_task", "graph_flow_llm_task")
        .add_edge("graph_flow_llm_task", "graph_flow_memory_store_task")
        .add_edge("graph_flow_memory_store_task", "graph_flow_response_post_process_task")
        .build(),
    );

    // 保存图定义
    self.graph_storage.save("ai_agent_workflow".to_string(), graph.clone()).await?;

    // 创建会话
    let session_id = format!("agent_session_{}", config.session_id);
    let session = Session::new_from_task(session_id.clone(), "graph_flow_message_preparation_task");

    // 设置会话上下文
    session.context.set("agent_config", config.clone()).await;
    session.context.set("user_input", user_input).await;
    session.context.set("workflow_id", "ai_agent_workflow".to_string()).await;

    // 保存会话
    self.session_storage.save(session.clone()).await?;

    // 执行工作流
    let mut final_result = None;
    loop {
      let execution_result = self.runner.run(&session_id).await?;

      match execution_result.status {
        ExecutionStatus::Completed => {
          info!("AI Agent workflow completed successfully");
          if let Some(response) = execution_result.response {
            final_result = Some(response);
          }
          break;
        }
        ExecutionStatus::WaitingForInput => {
          debug!("Workflow waiting for input, continuing...");
          continue;
        }
        ExecutionStatus::Paused { next_task_id, reason } => {
          info!("Workflow paused, continuing to task: {} (reason: {})", next_task_id, reason);
          continue;
        }
        ExecutionStatus::Error(e) => {
          warn!("AI Agent workflow failed: {}", e);
          return Err(format!("Workflow execution failed: {}", e).into());
        }
      }
    }

    // 提取最终响应
    if let Some(result) = final_result {
      let result_json: JsonValue = serde_json::from_str(&result.to_string())?;

      // 如果结果包含最终响应，直接返回
      if let Some(final_response) = result_json.get("final_response") {
        Ok(final_response.clone())
      } else {
        // 否则返回整个结果
        Ok(result_json)
      }
    } else {
      Err("No result from AI Agent workflow".into())
    }
  }

  /// 获取会话历史
  pub async fn get_session_history(
    &self,
    session_id: &str,
    count: usize,
  ) -> Result<Vec<GraphFlowConversationMessage>, Box<dyn std::error::Error + Send + Sync>> {
    self.memory_manager.retrieve_messages(session_id, count).await
  }

  /// 清空会话历史
  pub async fn clear_session_history(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    self.memory_manager.clear_memory(session_id).await
  }

  /// 获取内存统计信息
  pub async fn get_memory_stats(
    &self,
    session_id: &str,
  ) -> Result<Option<crate::memory::graph_flow_memory::GraphFlowMemoryStats>, Box<dyn std::error::Error + Send + Sync>>
  {
    self.memory_manager.get_memory_stats(session_id).await
  }
}

impl Default for GraphFlowAgentManager {
  fn default() -> Self {
    Self::new()
  }
}
