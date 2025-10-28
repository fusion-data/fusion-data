//! Graph-flow Integration Example
//!
//! 演示如何使用重构后的基于 graph-flow 的 AI Agent、Memory 和 LLM 节点
//! 展示完整的工作流：内存管理 -> LLM 调用 -> 响应处理

use std::env;
use std::sync::Arc;

use hetumind_nodes::cluster::ai_agent::{GraphFlowAgentConfig, GraphFlowAgentManager};
use hetumind_nodes::llm::deepseek_node::{GraphFlowDeepSeekConfig, GraphFlowDeepSeekManager};
use hetumind_nodes::memory::graph_flow_memory::{GraphFlowMemoryConfig, GraphFlowMemoryManager};
use log::{info, warn};
use serde_json::json;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  // 初始化日志

  info!("Starting Graph-flow Integration Example");

  // 从环境变量获取 API 密钥
  let api_key = env::var("DEEPSEEK_API_KEY").map_err(|_| "DEEPSEEK_API_KEY environment variable is required")?;

  // 创建管理器
  let memory_manager = Arc::new(GraphFlowMemoryManager::new());
  let deepseek_manager = GraphFlowDeepSeekManager::new();
  let agent_manager = GraphFlowAgentManager::new();

  // 创建会话ID
  let session_id = format!("example_session_{}", uuid::Uuid::new_v4());

  // 示例1: 独立使用内存管理器
  info!("\n=== Example 1: Memory Management ===");
  test_memory_management(&memory_manager, &session_id).await?;

  // 示例2: 独立使用 DeepSeek LLM
  info!("\n=== Example 2: DeepSeek LLM Call ===");
  test_deepseek_llm(&deepseek_manager, &session_id, &api_key).await?;

  // 示例3: 完整的 AI Agent 工作流
  info!("\n=== Example 3: Complete AI Agent Workflow ===");
  test_complete_agent_workflow(&agent_manager, &session_id, &api_key).await?;

  info!("\nGraph-flow Integration Example completed successfully!");
  Ok(())
}

/// 测试内存管理功能
async fn test_memory_management(
  memory_manager: &Arc<GraphFlowMemoryManager>,
  session_id: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
  info!("Testing memory management for session: {}", session_id);

  // 准备测试消息
  let test_messages = vec![
    json!({
      "role": "user",
      "content": "Hello, I want to ask a question about Rust programming.",
      "timestamp": chrono::Utc::now().to_rfc3339(),
    }),
    json!({
      "role": "assistant",
      "content": "Hello! I'd be happy to help you with Rust programming. What would you like to know?",
      "timestamp": chrono::Utc::now().to_rfc3339(),
    }),
  ];

  // 存储消息
  let memory_config = Some(GraphFlowMemoryConfig {
    session_id: session_id.to_string(),
    context_window_length: 10,
    persistence_enabled: false,
    input_key: "input".to_string(),
    memory_key: "chat_history".to_string(),
    output_key: "output".to_string(),
  });

  let memory_data = memory_manager.store_messages(session_id, "test_workflow", test_messages, memory_config).await?;

  info!("Stored {} messages in memory", memory_data.len());

  // 检索消息
  let retrieved_messages = memory_manager.retrieve_messages(session_id, 5).await?;
  info!("Retrieved {} messages from memory", retrieved_messages.len());

  for (i, msg) in retrieved_messages.iter().enumerate() {
    info!("Message {}: [{}] {}", i + 1, msg.role, msg.content);
  }

  // 获取统计信息
  if let Some(stats) = memory_manager.get_memory_stats(session_id).await? {
    info!("Memory stats - Total messages: {}, Context window: {}", stats.total_messages, stats.context_window_length);
  }

  Ok(())
}

/// 测试 DeepSeek LLM 调用
async fn test_deepseek_llm(
  deepseek_manager: &GraphFlowDeepSeekManager,
  session_id: &str,
  api_key: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
  info!("Testing DeepSeek LLM for session: {}", session_id);

  // 创建 DeepSeek 配置
  let deepseek_config = GraphFlowDeepSeekConfig {
    model: "deepseek-chat".to_string(),
    max_tokens: Some(1000),
    temperature: Some(0.7),
    top_p: None,
    stop_sequences: None,
    common: Default::default(),
    workflow_id: "test_workflow".to_string(),
    session_id: session_id.to_string(),
  };

  // 准备输入
  let input = json!({
    "prompt": "What is Rust programming language and what are its main advantages?",
    "session_id": session_id,
  });

  // 执行 LLM 调用
  match deepseek_manager.execute_llm_call(deepseek_config, input, api_key.to_string()).await {
    Ok(response) => {
      info!("DeepSeek LLM call successful!");

      if let Some(content) = response.get("content").and_then(|v| v.as_str()) {
        info!("Response: {}", content);
      }

      if let Some(model) = response.get("model").and_then(|v| v.as_str()) {
        info!("Model used: {}", model);
      }

      if let Some(usage) = response.get("usage") {
        info!("Usage statistics: {}", usage);
      }
    }
    Err(e) => {
      warn!("DeepSeek LLM call failed: {}", e);
      // 对于示例，我们继续执行，不因 API 密钥问题而中断
    }
  }

  Ok(())
}

/// 测试完整的 AI Agent 工作流
async fn test_complete_agent_workflow(
  agent_manager: &GraphFlowAgentManager,
  session_id: &str,
  api_key: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
  info!("Testing complete AI Agent workflow for session: {}", session_id);

  // 创建 AI Agent 配置
  let agent_config = GraphFlowAgentConfig {
    base_config: hetumind_nodes::cluster::ai_agent::parameters::AiAgentConfig {
      system_prompt: Some("You are a helpful AI assistant specializing in Rust programming. Provide clear, concise, and accurate answers.".to_string()),
      max_iterations: Some(3),
      temperature: Some(0.7),
      enable_streaming: Some(false),
      memory_config: Some(hetumind_nodes::cluster::ai_agent::parameters::MemoryConfig {
        max_history: Some(10),
        persistence_enabled: Some(false),
        context_window: Some(5),
      }),
    },
    session_id: session_id.to_string(),
    memory_config: Some(GraphFlowMemoryConfig {
      session_id: session_id.to_string(),
      context_window_length: 5,
      persistence_enabled: false,
      input_key: "input".to_string(),
      memory_key: "chat_history".to_string(),
      output_key: "output".to_string(),
    }),
    llm_config: Some(json!({
      "model": "deepseek-chat",
      "temperature": 0.7,
    })),
    tools_config: None, // 本示例不使用工具
  };

  // 准备用户输入
  let user_input = json!({
    "content": "What are the main benefits of using Rust for systems programming?",
    "role": "user",
  });

  // 执行 AI Agent 工作流
  match agent_manager
    .execute_agent_workflow(agent_config.clone(), user_input.clone(), api_key.to_string(), "deepseek-chat".to_string())
    .await
  {
    Ok(response) => {
      info!("AI Agent workflow completed successfully!");

      if let Some(content) = response.get("response").and_then(|v| v.as_str()) {
        info!("Agent response: {}", content);
      }

      if let Some(session_id) = response.get("session_id").and_then(|v| v.as_str()) {
        info!("Session ID: {}", session_id);
      }

      // 检查会话历史
      let history = agent_manager.get_session_history(session_id, 10).await?;
      info!("Session history contains {} messages", history.len());

      // 添加另一个问题来测试对话上下文
      info!("\n--- Follow-up Question ---");
      let follow_up_input = json!({
        "content": "Can you explain Rust's ownership system in more detail?",
        "role": "user",
      });

      match agent_manager
        .execute_agent_workflow(agent_config, follow_up_input, api_key.to_string(), "deepseek-chat".to_string())
        .await
      {
        Ok(follow_up_response) => {
          if let Some(content) = follow_up_response.get("response").and_then(|v| v.as_str()) {
            info!("Follow-up response: {}", content);
          }
        }
        Err(e) => {
          warn!("Follow-up workflow failed: {}", e);
        }
      }
    }
    Err(e) => {
      warn!("AI Agent workflow failed: {}", e);
      // 对于示例，我们继续执行
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_memory_manager_creation() {
    let memory_manager = GraphFlowMemoryManager::new();
    assert!(!format!("{:?}", memory_manager).is_empty());
  }

  #[tokio::test]
  async fn test_deepseek_manager_creation() {
    let deepseek_manager = GraphFlowDeepSeekManager::new();
    assert!(!format!("{:?}", deepseek_manager).is_empty());
  }

  #[tokio::test]
  async fn test_agent_manager_creation() {
    let agent_manager = GraphFlowAgentManager::new();
    assert!(!format!("{:?}", agent_manager).is_empty());
  }

  #[test]
  fn test_graph_flow_config_creation() {
    let memory_config = GraphFlowMemoryConfig::default();
    assert_eq!(memory_config.context_window_length, 5);
    assert!(!memory_config.session_id.is_empty());

    let deepseek_config = GraphFlowDeepSeekConfig::default();
    assert_eq!(deepseek_config.model, "deepseek-chat");
    assert!(!deepseek_config.session_id.is_empty());

    let agent_config = GraphFlowAgentConfig::default();
    assert!(!agent_config.session_id.is_empty());
  }
}
