//! DeepSeek Supplier 最小端到端驱动示例
//!
//! 运行前请确保环境变量 `DEEPSEEK_API_KEY` 已设置或在 .env 文件中声明。
//! 运行命令：
//!   cargo run -p hetumind-nodes --example deepseek_supplier_smoke
//!
//! 本示例直接调用 DeepseekModelSupplier 的 call_llm 接口，构造 Message 与 LLMConfig，
//! 验证 temperature/max_tokens 绑定，以及通过 additional_params 透传 top_p/stop（stop_sequences → stop）。

use hetumind_core::workflow::{LLMConfig, LLMResponse, LLMSubNodeProvider, Message, SubNode};
use hetumind_nodes::llm::deepseek_node::DeepseekModelSupplier;

/// 构造示例消息（system + user）
fn build_messages() -> Vec<Message> {
  vec![
    Message { role: "system".to_string(), content: "你是一个专业助理，请简洁准确回答。".to_string() },
    Message {
      role: "user".to_string(), content: "用两句话说明 Rust 的所有权模型，并在句末加上 END。".to_string()
    },
  ]
}

/// 构造 LLMConfig（绑定 temperature/max_tokens/top_p/stop_sequences），API Key 从环境变量解析
fn build_config() -> LLMConfig {
  LLMConfig {
    model: "deepseek-chat".to_string(),
    max_tokens: Some(256),
    temperature: Some(0.7),
    // LLMConfig.top_p 为 u32，这里示例使用 1（相当于 1.0）
    top_p: Some(1),
    // 将 stop_sequences 传入，Supplier 会转换为 additional_params 中的 "stop" 字段
    stop_sequences: Some(vec!["END".to_string()]),
    // 支持 ${env:VAR} 格式的环境变量解析
    api_key: Some("${env:DEEPSEEK_API_KEY}".to_string()),
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::dotenv().ok();

  // 1) 创建 Supplier
  let supplier = DeepseekModelSupplier::new();
  supplier.initialize().await?;

  // 2) 构造消息与配置
  let messages = build_messages();
  let config = build_config();

  // 3) 调用 LLM
  let resp: LLMResponse = supplier.call_llm(messages, config).await?;

  // 4) 打印输出（content + usage）
  println!("Supplier Output: {}", serde_json::to_string_pretty(&resp)?);

  Ok(())
}
