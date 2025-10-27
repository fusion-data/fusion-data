//! Graph-flow based DeepSeek LLM Implementation
//!
//! 基于 graph-flow 框架重构的 DeepSeek LLM 节点，将 LLM 调用封装为图流任务
//! 提供独立的 LLM 服务，支持与其他 graph-flow 节点的集成

use async_trait::async_trait;
use fusion_ai::graph_flow::{
  Context, ExecutionStatus, FlowRunner, GraphBuilder, GraphError, GraphStorage, InMemoryGraphStorage,
  InMemorySessionStorage, NextAction, Session, SessionStorage, Task, TaskResult,
};
use fusion_common::time::now_offset;
use hetumind_core::types::JsonValue;
use log::{debug, info, warn};
use rig::{
  client::CompletionClient,
  completion::Completion,
  message::{AssistantContent, Message},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::llm::shared::{CommonLlmParameters, ModelCapabilities, UsageStats};

/// 图流 DeepSeek LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GraphFlowDeepSeekConfig {
  /// 模型名称
  pub model: String,
  /// 最大令牌数
  pub max_tokens: Option<u32>,
  /// 温度参数
  pub temperature: Option<f64>,
  /// Top-p 参数
  pub top_p: Option<u32>,
  /// 停止序列
  pub stop_sequences: Option<Vec<String>>,
  /// 通用 LLM 参数
  pub common: CommonLlmParameters,
  /// 工作流ID
  pub workflow_id: String,
  /// 会话ID
  pub session_id: String,
}

impl Default for GraphFlowDeepSeekConfig {
  fn default() -> Self {
    Self {
      model: "deepseek-chat".to_string(),
      max_tokens: Some(128000),
      temperature: Some(0.7),
      top_p: None,
      stop_sequences: None,
      common: CommonLlmParameters::default(),
      workflow_id: Uuid::new_v4().to_string(),
      session_id: Uuid::new_v4().to_string(),
    }
  }
}

/// DeepSeek LLM 调用任务
pub struct GraphFlowDeepSeekLLMTask {
  /// API 密钥
  api_key: String,
  /// 模型配置
  config: GraphFlowDeepSeekConfig,
}

impl GraphFlowDeepSeekLLMTask {
  pub fn new(api_key: String, config: GraphFlowDeepSeekConfig) -> Self {
    Self { api_key, config }
  }
}

#[async_trait]
impl Task for GraphFlowDeepSeekLLMTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowDeepSeekLLMTask with model: {}", self.config.model);

    // 获取输入消息
    let input_data: JsonValue = context.get_sync("input_messages").unwrap_or_else(|| json!({}));

    debug!("Input data for DeepSeek LLM: {}", serde_json::to_string_pretty(&input_data).unwrap_or_default());

    // 解析消息
    let messages = self
      .parse_input_messages(&input_data)
      .map_err(|e| GraphError::TaskExecutionFailed(format!("Failed to parse input messages: {}", e)))?;

    if messages.is_empty() {
      warn!("No valid messages found for DeepSeek LLM call");
      let error_result = json!({
        "error": "No valid messages found for LLM call",
        "content": "",
        "role": "assistant",
        "model": self.config.model,
        "timestamp": now_offset(),
      });

      return Ok(TaskResult::new(Some(error_result.to_string()), NextAction::Continue));
    }

    // 创建 DeepSeek 客户端
    let client = rig::providers::deepseek::Client::new(&self.api_key);

    // 构建代理
    let mut agent_builder = client.agent(&self.config.model);

    // 应用配置参数
    if let Some(temperature) = self.config.temperature {
      agent_builder = agent_builder.temperature(temperature);
    }

    if let Some(max_tokens) = self.config.max_tokens {
      // rig-core 可能使用不同的参数名，这里需要根据实际情况调整
      // agent_builder = agent_builder.max_tokens(max_tokens);
    }

    let agent = agent_builder.build();

    debug!("Sending {} messages to DeepSeek LLM", messages.len());

    // 执行 LLM 调用
    match agent.completion(messages[0].clone(), messages[1..].to_vec()).await {
      Ok(completion) => {
        info!("DeepSeek LLM call successful");

        // 发送完成请求获取最终响应
        match completion.send().await {
          Ok(completion_response) => {
            let usage_stats = UsageStats::from(completion_response.usage);

            // 提取响应文本
            let choice = completion_response.choice.first();
            let response_text = match choice {
              AssistantContent::Text(text) => text.text.clone(),
              _ => "".to_string(),
            };

            // 创建模型能力信息
            let capabilities = ModelCapabilities {
              chat: true,
              completion: true,
              tools: true,
              streaming: true,
              function_calling: true,
              vision: false,
              max_context_length: Some(8192),
              supported_formats: vec!["text".to_string(), "json".to_string()],
              json_mode: true,
              system_messages: true,
              temperature_control: true,
            };

            let result = json!({
              "content": response_text,
              "role": "assistant",
              "model": self.config.model,
              "timestamp": now_offset(),
              "usage": {
                "prompt_tokens": usage_stats.prompt_tokens,
                "completion_tokens": usage_stats.completion_tokens,
                "total_tokens": usage_stats.total_tokens,
              },
              "capabilities": {
                "chat": capabilities.chat,
                "completion": capabilities.completion,
                "tools": capabilities.tools,
                "streaming": capabilities.streaming,
                "function_calling": capabilities.function_calling,
                "vision": capabilities.vision,
                "max_context_length": capabilities.max_context_length,
                "supported_formats": capabilities.supported_formats,
                "json_mode": capabilities.json_mode,
                "system_messages": capabilities.system_messages,
                "temperature_control": capabilities.temperature_control,
              },
              "session_id": self.config.session_id,
              "workflow_id": self.config.workflow_id,
            });

            Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
          }
          Err(e) => {
            warn!("DeepSeek LLM completion send failed: {}", e);
            let error_result = json!({
              "error": format!("Completion send failed: {}", e),
              "content": "",
              "role": "assistant",
              "model": self.config.model,
              "timestamp": now_offset(),
            });

            Ok(TaskResult::new(Some(error_result.to_string()), NextAction::Continue))
          }
        }
      }
      Err(e) => {
        warn!("DeepSeek LLM call failed: {}", e);
        let error_result = json!({
          "error": format!("LLM call failed: {}", e),
          "content": "",
          "role": "assistant",
          "model": self.config.model,
          "timestamp": now_offset(),
        });

        Ok(TaskResult::new(Some(error_result.to_string()), NextAction::Continue))
      }
    }
  }
}

impl GraphFlowDeepSeekLLMTask {
  /// 解析输入消息
  fn parse_input_messages(&self, input_data: &JsonValue) -> Result<Vec<Message>, String> {
    let mut messages = Vec::new();

    // 尝试不同的输入格式
    if let Some(prompt) = input_data.get("prompt").and_then(|v| v.as_str()) {
      // 单个提示词格式
      messages.push(Message::user(prompt.to_string()));
    } else if let Some(content) = input_data.get("content").and_then(|v| v.as_str()) {
      // 单个内容格式
      messages.push(Message::user(content.to_string()));
    } else if let Some(msg_array) = input_data.get("messages").and_then(|v| v.as_array()) {
      // 消息数组格式
      for msg in msg_array {
        if let (Some(role), Some(content)) =
          (msg.get("role").and_then(|v| v.as_str()), msg.get("content").and_then(|v| v.as_str()))
        {
          match role {
            "user" => messages.push(Message::user(content.to_string())),
            "assistant" => messages.push(Message::assistant(content.to_string())),
            _ => {
              debug!("Unknown message role: {}, treating as user", role);
              messages.push(Message::user(content.to_string()));
            }
          }
        }
      }
    }

    Ok(messages)
  }
}

/// DeepSeek LLM 预处理任务
pub struct GraphFlowDeepSeekPreprocessTask;

#[async_trait]
impl Task for GraphFlowDeepSeekPreprocessTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowDeepSeekPreprocessTask");

    // 获取配置
    let config: GraphFlowDeepSeekConfig = context.get_sync("deepseek_config").unwrap_or_default();

    // 获取原始输入
    let raw_input: JsonValue = context.get_sync("raw_input").unwrap_or_else(|| json!({}));

    debug!("Raw input for DeepSeek preprocessing: {}", serde_json::to_string_pretty(&raw_input).unwrap_or_default());

    // 验证和处理输入
    let processed_input = self.validate_and_process_input(&raw_input, &config)?;

    let result = json!({
      "input_messages": processed_input,
      "config": config,
      "timestamp": now_offset(),
    });

    Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
  }
}

impl GraphFlowDeepSeekPreprocessTask {
  /// 验证和处理输入
  fn validate_and_process_input(
    &self,
    raw_input: &JsonValue,
    _config: &GraphFlowDeepSeekConfig,
  ) -> fusion_ai::graph_flow::Result<JsonValue> {
    // 这里可以添加输入验证和预处理逻辑
    // 目前直接返回原始输入
    Ok(raw_input.clone())
  }
}

/// DeepSeek LLM 后处理任务
pub struct GraphFlowDeepSeekPostprocessTask;

#[async_trait]
impl Task for GraphFlowDeepSeekPostprocessTask {
  async fn run(&self, context: Context) -> fusion_ai::graph_flow::Result<TaskResult> {
    info!("Running GraphFlowDeepSeekPostprocessTask");

    // 获取 LLM 响应
    let llm_response: JsonValue = context.get_sync("llm_response").unwrap_or_else(|| json!({}));

    // 获取配置
    let config: GraphFlowDeepSeekConfig = context.get_sync("deepseek_config").unwrap_or_default();

    // 格式化最终输出
    let formatted_response = self
      .format_response(&llm_response, &config)
      .map_err(|e| GraphError::TaskExecutionFailed(format!("Failed to format response: {}", e)))?;

    let result = json!({
      "response": formatted_response,
      "raw_response": llm_response,
      "model_info": {
        "name": config.model,
        "provider": "deepseek",
        "capabilities": self.get_model_capabilities(),
      },
      "session_id": config.session_id,
      "timestamp": now_offset(),
    });

    Ok(TaskResult::new(Some(result.to_string()), NextAction::Continue))
  }
}

impl GraphFlowDeepSeekPostprocessTask {
  /// 格式化响应
  fn format_response(&self, llm_response: &JsonValue, _config: &GraphFlowDeepSeekConfig) -> Result<JsonValue, String> {
    // 这个方法现在总是返回 JsonValue，简化错误处理
    Ok(self.create_formatted_response(llm_response))
  }

  fn create_formatted_response(&self, llm_response: &JsonValue) -> JsonValue {
    // 提取主要内容
    let content = llm_response.get("content").and_then(|v| v.as_str()).unwrap_or("");

    // 构建格式化的响应
    json!({
      "content": content,
      "role": "assistant",
      "model": llm_response.get("model").unwrap_or(&json!("deepseek-chat")),
      "timestamp": llm_response.get("timestamp").unwrap_or(&json!(now_offset().to_rfc3339())),
      "usage": llm_response.get("usage").unwrap_or(&json!({})),
    })
  }

  /// 获取模型能力信息
  fn get_model_capabilities(&self) -> JsonValue {
    json!({
      "chat": true,
      "completion": true,
      "tools": true,
      "streaming": true,
      "function_calling": true,
      "vision": false,
      "max_context_length": 8192,
      "supported_formats": ["text", "json"],
      "json_mode": true,
      "system_messages": true,
      "temperature_control": true,
    })
  }
}

/// 图流 DeepSeek LLM 管理器
#[derive(Clone)]
pub struct GraphFlowDeepSeekManager {
  /// 会话存储
  session_storage: Arc<dyn SessionStorage>,
  /// 图存储
  graph_storage: Arc<dyn GraphStorage>,
  /// 工作流运行器
  runner: FlowRunner,
}

impl GraphFlowDeepSeekManager {
  /// 创建新的 DeepSeek LLM 管理器
  pub fn new() -> Self {
    let session_storage: Arc<dyn SessionStorage> = Arc::new(InMemorySessionStorage::new());
    let graph_storage: Arc<dyn GraphStorage> = Arc::new(InMemoryGraphStorage::new());

    // 创建一个简单的空图用于 DeepSeek LLM 管理
    let graph = Arc::new(GraphBuilder::new("deepseek_graph").build());

    Self { session_storage: session_storage.clone(), graph_storage, runner: FlowRunner::new(graph, session_storage) }
  }

  /// 执行 DeepSeek LLM 调用
  pub async fn execute_llm_call(
    &self,
    config: GraphFlowDeepSeekConfig,
    input: JsonValue,
    api_key: String,
  ) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>> {
    info!("Executing DeepSeek LLM call with model: {}", config.model);

    // 创建工作流图
    let graph = Arc::new(
      GraphBuilder::new("deepseek_llm_workflow")
        .add_task(Arc::new(GraphFlowDeepSeekPreprocessTask))
        .add_task(Arc::new(GraphFlowDeepSeekLLMTask::new(api_key, config.clone())))
        .add_task(Arc::new(GraphFlowDeepSeekPostprocessTask))
        .add_edge("graph_flow_deepseek_preprocess_task", "graph_flow_deepseek_llm_task")
        .add_edge("graph_flow_deepseek_llm_task", "graph_flow_deepseek_postprocess_task")
        .build(),
    );

    // 保存图定义
    self.graph_storage.save("deepseek_llm_workflow".to_string(), graph.clone()).await?;

    // 创建会话
    let session_id = format!("deepseek_session_{}", config.session_id);
    let session = Session::new_from_task(session_id.clone(), "graph_flow_deepseek_preprocess_task");

    // 设置会话上下文
    session.context.set("deepseek_config", config.clone()).await;
    session.context.set("raw_input", input).await;

    // 保存会话
    self.session_storage.save(session.clone()).await?;

    // 执行工作流
    let mut final_result = None;
    loop {
      let execution_result = self.runner.run(&session_id).await?;

      match execution_result.status {
        ExecutionStatus::Completed => {
          info!("DeepSeek LLM workflow completed successfully");
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
          warn!("DeepSeek LLM workflow failed: {}", e);
          return Err(format!("Workflow execution failed: {}", e).into());
        }
      }
    }

    // 提取最终响应
    if let Some(result) = final_result {
      let result_json: JsonValue = serde_json::from_str(&result.to_string())?;

      // 如果结果包含格式化响应，直接返回
      if let Some(response) = result_json.get("response") {
        Ok(response.clone())
      } else {
        // 否则返回整个结果
        Ok(result_json)
      }
    } else {
      Err("No result from DeepSeek LLM workflow".into())
    }
  }

  /// 流式调用 DeepSeek LLM（当前实现为非流式）
  pub async fn stream_llm_call(
    &self,
    _config: GraphFlowDeepSeekConfig,
    _input: JsonValue,
    _api_key: String,
  ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: 实现流式调用
    // 当前返回错误，表示流式调用尚未实现
    Err("Streaming not implemented for GraphFlow DeepSeek LLM".into())
  }
}

impl Default for GraphFlowDeepSeekManager {
  fn default() -> Self {
    Self::new()
  }
}
