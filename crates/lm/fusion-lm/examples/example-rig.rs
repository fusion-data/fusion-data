use rig::{
  Embed, completion::Prompt, embeddings::EmbeddingsBuilder, providers,
  vector_store::in_memory_store::InMemoryVectorStore,
};
use serde::Serialize;

// 需要进行 RAG 处理的数据。需要对 `definitions` 字段执行向量搜索，
// 因此我们为 `WordDefinition` 标记 `#[embed]` 宏以派生 `Embed` trait。
#[derive(Embed, Serialize, Clone, Debug, Eq, PartialEq, Default)]
struct WordDefinition {
  id: String,
  word: String,
  #[embed]
  definitions: Vec<String>,
}

/// 运行示例
/// ```shell
/// cargo run -p llm-box --example example-rag
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
  const MODEL_NAME: &str = "qwen2.5";
  // const MODEL_NAME: &str = "deepseek-r1";
  // const MODEL_NAME: &str = "deepseek-llm";
  // const EMBEDDING_MODEL: &str = TEXT_EMBEDDING_ADA_002;
  // const EMBEDDING_MODEL: &str = "nomic-embed-text";
  const EMBEDDING_MODEL: &str = "aroxima/gte-qwen2-1.5b-instruct";

  let client = providers::openai::Client::from_url("ollama", "http://localhost:11434/v1");

  let embedding_model = client.embedding_model(EMBEDDING_MODEL);

  // 使用指定的嵌入模型为所有文档的定义生成嵌入向量
  let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
    .documents(vec![
      WordDefinition {
        id: "doc0".to_string(),
        word: "flurbo".to_string(),
        definitions: vec![
          "1. *flurbo* （名词）：flurbo是一种生活在寒冷行星上的绿色外星人。".to_string(),
          "2. *flurbo* （名词）：一种虚构的数字货币，起源于动画系列《瑞克和莫蒂》。".to_string(),
        ],
      },
      WordDefinition {
        id: "doc1".to_string(),
        word: "glarb glarb".to_string(),
        definitions: vec![
          "1. *glarb glarb* （名词）：glarb glarb是次郎星球居民祖先用来耕种土地的古老工具。".to_string(),
          "2. *glarb glarb* （名词）：一种虚构的生物，发现于仙女座星系Glibbo星球遥远的沼泽地。".to_string(),
        ],
      },
      WordDefinition {
        id: "doc2".to_string(),
        word: "linglingdong".to_string(),
        definitions: vec![
          "1. *linglingdong* （名词）：月球背面的居民用来描述人类的术语。".to_string(),
          "2. *linglingdong* （名词）：一种罕见的神秘乐器，由夸姆星球内布隆山脉的古代僧侣制作。".to_string(),
        ],
      },
    ])?
    .build()
    .await?;

  // 使用这些嵌入创建向量存储
  let vector_store = InMemoryVectorStore::from_documents(embeddings);

  // 创建向量存储索引
  let index = vector_store.index(embedding_model);

  let rag_agent = client
    .agent(MODEL_NAME)
    .preamble("您是这里的词典助理，帮助用户理解单词的含义。\n您将在下面找到其他可能有用的非标准单词定义。")
    .dynamic_context(1, index)
    .build();

  // 提示并打印响应
  let response = rag_agent.prompt("\"glarb glarb\" 是什么意思？").await?;

  println!("{}", response);

  Ok(())
}
