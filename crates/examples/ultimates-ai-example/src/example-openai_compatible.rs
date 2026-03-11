use rig::completion::Completion;
use ultimates::ai::{
  AiError, DefaultProviders,
  client::{AgentConfigBuilder, ClientFactory},
  providers::openai_compatible::CompletionModel,
};

/// 示例：使用 OpenAI 兼容 API 调用模型
///
/// `RUST_LOG=debug cargo run -p fusion-ai --example example-openai_compatible`
#[tokio::main]
async fn main() -> Result<(), AiError> {
  dotenvy::dotenv().unwrap();
  logforth::starter_log::stdout().apply();

  let config = AgentConfigBuilder::default()
    .provider(DefaultProviders::OPENAI_COMPATIBLE)
    .name("Openai Compatible Agent")

    // .base_url("https://open.bigmodel.cn/api/coding/paas/v4")
    // .api_key(std::env::var("ZAI_API_KEY").unwrap())
    // .model("glm-4.6")

    // .base_url("https://api.deepseek.com/v1")
    // .api_key(std::env::var("DEEPSEEK_API_KEY").unwrap())
    // .model("deepseek-chat")

    // .base_url("https://api.siliconflow.cn/v1")
    // .api_key(std::env::var("SILICONFLOW_API_KEY").unwrap())
    // .model("deepseek-ai/DeepSeek-OCR")

    .base_url("https://ai.gitee.com/v1")
    .api_key(std::env::var("GITEE_AI_API_KEY").unwrap())
    .model("Kimi-K2-Thinking")

    .description("使用 Fusion AI 的示例 AI Agent")
    .system_prompt("你是一个 AI 助手")
    .temperature(0.7)
    // .max_tokens(248000)
    .build()
    .unwrap();

  let factory =
    ClientFactory::new().openai_compatible(config.base_url.as_ref().unwrap(), config.api_key.as_ref().unwrap());
  let agent = CompletionModel::new(factory.to_inner_cloned(), &config.model).into_agent_builder().build();

  let request = agent.completion("你是谁？", vec![]).await?;
  let response = request.send().await?;

  println!("Response usage: {}", serde_json::to_string_pretty(&response.usage).unwrap());
  println!("Response choice: {}", serde_json::to_string_pretty(&response.choice).unwrap());
  Ok(())
}
