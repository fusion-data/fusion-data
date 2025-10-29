//! 集成测试通用工具模块
//!
//! 提供测试所需的通用功能和工具类

use std::sync::Arc;
use std::time::Duration;

use config::{File, FileFormat};
use tokio::time::sleep;

use hetuflow_agent::application::AgentApplication;
use hetuflow_agent::setting::HetuflowAgentSetting;
use hetuflow_core::types::Labels;
use hetuflow_server::application::ServerApplication;
use hetuflow_server::setting::HetuflowSetting;

/// Server 端测试上下文
pub struct ServerTestContext {
  pub server: ServerApplication,
  pub server_setting: Arc<HetuflowSetting>,
}

/// Agent 端测试上下文
pub struct AgentTestContext {
  pub agents: Vec<AgentApplication>,
  pub agent_settings: Vec<Arc<HetuflowAgentSetting>>,
}

/// 启动测试环境
pub async fn setup_test_environment() -> anyhow::Result<ServerTestContext> {
  // 创建服务器配置

  // 启动服务器
  let server = ServerApplication::new_with_source(Some(create_test_server_config_source())).await?;
  server.start().await?;

  // 等待服务器启动
  sleep(Duration::from_secs(2)).await;

  let server_setting = server.setting.clone();
  Ok(ServerTestContext { server, server_setting })
}

pub async fn setup_agent_test_context() -> anyhow::Result<AgentTestContext> {
  // 创建测试Agent配置
  let agent_config_sources = vec![
    create_test_agent_config_source("agent-1", Labels::from([("env", "test"), ("region", "us-west")])),
    create_test_agent_config_source("agent-2", Labels::from([("env", "test"), ("region", "us-east")])),
  ];

  // 启动Agent
  let mut agents = Vec::new();
  for config in agent_config_sources {
    let agent = AgentApplication::new_with_source(Some(config)).await?;
    let agent_handle = agent.clone();
    tokio::spawn(async move {
      if let Err(e) = agent_handle.start().await {
        eprintln!("Agent error: {}", e);
      }
    });
    agents.push(agent);
  }

  // 等待Agent启动和注册
  sleep(Duration::from_secs(3)).await;

  let agent_settings = agents.iter().map(|a| a.setting.clone()).collect();
  Ok(AgentTestContext { agents, agent_settings })
}

/// 清理测试环境
pub async fn cleanup_test_environment(ctx: AgentTestContext) -> anyhow::Result<()> {
  for agent in ctx.agents {
    agent.shutdown().await?;
  }
  Ok(())
}

pub async fn cleanup_server_text_context(ctx: ServerTestContext) -> anyhow::Result<()> {
  ctx.server.shutdown_and_await().await?;
  Ok(())
}

/// 等待条件满足
pub async fn wait_for_condition<F, Fut>(condition: F, timeout: Duration, check_interval: Duration) -> anyhow::Result<()>
where
  F: Fn() -> Fut,
  Fut: std::future::Future<Output = bool>,
{
  let start = std::time::Instant::now();

  while start.elapsed() < timeout {
    if condition().await {
      return Ok(());
    }
    sleep(check_interval).await;
  }

  anyhow::bail!("Condition not met within timeout")
}

/// 创建测试用的服务器配置
fn create_test_server_config_source() -> File<config::FileSourceString, FileFormat> {
  let c = r#"
"#;
  File::from_str(c, FileFormat::Toml)
}

/// 创建测试用的Agent配置
fn create_test_agent_config_source(agent_id: &str, labels: Labels) -> config::Config {
  let mut cb = config::Config::builder();
  cb = cb.set_override("hetuflow.agent.id", agent_id).unwrap();
  for (key, value) in labels.into_inner() {
    cb = cb.set_override(format!("hetuflow.agent.labels.{}", key), value.as_str()).unwrap();
  }
  cb.build().unwrap()
}
