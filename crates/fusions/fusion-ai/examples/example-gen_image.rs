// Respond url
// .base_url("https://api.siliconflow.cn/v1")
// .api_key(std::env::var("SILICONFLOW_API_KEY").unwrap())
// .model("Kwai-Kolors/Kolors")

// Respond url
// .base_url("https://open.bigmodel.cn/api/coding/paas/v4")
// .api_key(std::env::var("ZAI_API_KEY").unwrap())
// .model("cogview-4-250304")

use fusion_ai::{
  AiError, DefaultProviders, agents::AgentConfigBuilder, client::ClientBuilderFactory, utils::vec_to_image_file,
};
#[allow(unused_imports)]
use serde_json::json;

/// 生成图片示例
///
/// `RUST_LOG=debug cargo run -p fusion-ai --example example-gen_image`
#[tokio::main]
async fn main() -> Result<(), AiError> {
  dotenvy::dotenv().unwrap();
  logforth::starter_log::stdout().apply();

  let config = AgentConfigBuilder::default()
    .provider(DefaultProviders::OPENAI_COMPATIBLE)

    // Respond url or b64_json (when response_format is b64_json)
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
    .height(1024);
  // .additional_params(json!({"response_format": "b64_json"}));
  let response = request.send().await?;
  println!("Response image bytes: {} | {:?}", response.image.len(), &response.image[..50]);

  // 将生成的图片保存到文件
  let output_path = "runs/generated_image.png";
  vec_to_image_file(&response.image, output_path)
    .map_err(|e| AiError::Custom(format!("Failed to save image: {}", e)))?;

  println!("图片已生成并保存到: {}", output_path);

  Ok(())
}
