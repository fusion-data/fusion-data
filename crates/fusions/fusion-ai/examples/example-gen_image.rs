use fusion_ai::{AiError, DefaultProviders, agents::AgentConfigBuilder, client::ClientBuilderFactory};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), AiError> {
  dotenvy::dotenv().unwrap();
  logforth::starter_log::stdout().apply();

  let config = AgentConfigBuilder::default()
    .provider(DefaultProviders::OPENAI_COMPATIBLE)

    // 注意： siliconflow 的 image 模型响应不是 openai 兼容格式
    // .base_url("https://api.siliconflow.cn/v1")
    // .api_key(std::env::var("SILICONFLOW_API_KEY").unwrap())
    // .model("Kwai-Kolors/Kolors")

    // .base_url("https://open.bigmodel.cn/api/coding/paas/v4")
    // .api_key(std::env::var("ZAI_API_KEY").unwrap())
    // .model("cogview-4-250304")

    .base_url("https://ai.gitee.com/v1")
    .api_key(std::env::var("GITEE_AI_API_KEY").unwrap())
    .model("flux-1-schnell")

    .build()
    .unwrap();

  let factory = ClientBuilderFactory::new();
  let agent = factory.image(&config)?;

  let request = agent.image_generation_request()
    .prompt("使用 Rust, Python, Typescript 这 3 门编程语言的 logo 合成一个新的 logo。要求新 logo 能够让专业人士明确的分辨出包含有这 3 门编程语言的元素")
    .width(1024)
    .height(1024)
    .additional_params(json!({
      "response_format": "b64_json"
    }));
  let response = request.send().await?;
  println!("Response: {:?}", response);

  Ok(())
}
