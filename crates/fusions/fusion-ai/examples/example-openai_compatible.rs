use fusion_ai::{AiError, DefaultProviders, agents::AgentConfigBuilder, client::ClientBuilderFactory};
use rig::completion::Completion;

#[tokio::main]
async fn main() -> Result<(), AiError> {
  dotenvy::dotenv().unwrap();
  logforth::starter_log::stdout().apply();

  let config = AgentConfigBuilder::default()
    .provider(DefaultProviders::OPENAI_COMPATIBLE)
    .base_url("https://open.bigmodel.cn/api/coding/paas/v4")
    .api_key(std::env::var("ZAI_API_KEY").unwrap())
    .model("glm-4.6")
    // .base_url("https://api.deepseek.com/v1")
    // .api_key(std::env::var("DEEPSEEK_API_KEY").unwrap())
    // .model("deepseek-chat")
    // .name("Z AI Agent")
    .description("使用 Fusion AI 的示例 AI Agent")
    .system_prompt("你是一个 AI 助手")
    .temperature(0.7)
    // .max_tokens(248000)
    .build()
    .unwrap();

  let factory = ClientBuilderFactory::new();
  let agent = factory.agent(&config)?;

  let request = agent.completion("你是谁？", vec![]).await?;
  let response = request.send().await?;

  println!("Response: {:?}", response);
  Ok(())
}
