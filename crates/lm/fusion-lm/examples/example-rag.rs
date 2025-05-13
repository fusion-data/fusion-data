use rig::{Embed, completion::Prompt, embeddings::EmbeddingsBuilder, providers};
use rig_postgres::PostgresVectorStore;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::{env, vec};

// 需要进行 RAG 处理的数据。需要对 `content` 字段执行向量搜索，
// 因此我们为 `DocumentChunk` 标记 `#[embed]` 宏以派生 `Embed` trait。
#[derive(Embed, Serialize, Clone, Debug, PartialEq, Default)]
struct DocumentChunk {
  id: String,
  title: String,
  #[embed]
  content: String,
}

/// 运行示例
/// ```shell
/// cargo run -p fusion-lm --example example-rag
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // 加载环境变量
  dotenvy::dotenv().ok();

  // 模型配置
  const MODEL_NAME: &str = "qwen2.5";
  const EMBEDDING_MODEL: &str = "aroxima/gte-qwen2-1.5b-instruct";

  // 创建 Ollama 客户端
  let client = providers::openai::Client::from_url("ollama", "http://localhost:11434/v1");
  let embedding_model = client.embedding_model(EMBEDDING_MODEL);

  // 准备示例文档数据
  let chunks = vec![
        DocumentChunk {
            id: "doc1".to_string(),
            title: "协作笔记系统概述".to_string(),
            content: "协作笔记系统是一款支持富文本编辑、数据库管理、团队协作的轻量化平台，提供类似 Notion 和 AFFiNE 的功能，同时强化本地优先存储和国内云服务访问体验。".to_string(),
        },
        DocumentChunk {
            id: "doc2".to_string(),
            title: "核心目标".to_string(),
            content: "系统的核心目标包括：提供模块化编辑与页面嵌套能力，实现本地优先存储，支持多人实时协作与权限管理，实现跨平台同步，集成AI能力，优化国内云服务访问体验。".to_string(),
        },
        DocumentChunk {
            id: "doc3".to_string(),
            title: "技术架构".to_string(),
            content: "后端采用Axum框架，使用gRPC进行服务间通信，PostgreSQL作为数据库并启用pgvector插件支持向量计算，采用y-octo作为CRDT算法库实现实时协作。".to_string(),
        },
        DocumentChunk {
            id: "doc4".to_string(),
            title: "AI能力集成".to_string(),
            content: "系统集成了多种AI功能，包括文本续写（使用Qwen/Deepseek等模型），文档摘要（BART/Longformer模型），图片解析（ResNet/ViT模型），以及基于pgvector的内容推荐。".to_string(),
        },
        DocumentChunk {
            id: "doc5".to_string(),
            title: "数据存储与同步".to_string(),
            content: "采用本地优先存储策略，使用IndexedDB存储文档数据，y-octo处理并发编辑，实现增量同步和完整离线编辑能力，自动合并冲突。".to_string(),
        },
    ];

  // 获取数据库连接字符串
  let database_url = env::var("DATABASE_URL")
    .unwrap_or_else(|_| "postgres://fusiondata:2024.Fusiondata@localhost:45432/fusiondata".to_string());

  // 创建数据库连接池
  let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

  // 确保表存在（如果不存在则创建）
  sqlx::query(
    "CREATE TABLE IF NOT EXISTS documents (
       id uuid DEFAULT gen_random_uuid(), -- we can have repeated entries
       document jsonb NOT NULL,
       embedded_text text NOT NULL,
       embedding vector(1536),
       ctime timestamptz default now() not null
     )",
  )
  .execute(&pool)
  .await?;

  // 创建向量索引（如果不存在）
  sqlx::query(
    "CREATE INDEX IF NOT EXISTS document_embeddings_idx ON documents
     USING hnsw(embedding vector_cosine_ops);",
  )
  .execute(&pool)
  .await?;

  // 使用指定的嵌入模型为所有文档生成嵌入向量
  let documents = EmbeddingsBuilder::new(embedding_model.clone()).documents(chunks)?.build().await?;

  // 创建PostgreSQL向量存储
  // "id", "embedding", Some("content")
  let vector_store = PostgresVectorStore::with_defaults(embedding_model, pool);

  // 将嵌入向量存储到数据库
  vector_store.insert_documents(documents).await?;

  // 创建向量存储索引
  // let index = vector_store.index(embedding_model);

  // 创建RAG代理
  let rag_agent = client
    .agent(MODEL_NAME)
    .preamble("您是一位协作知识库系统专家，可以回答关于系统设计和功能的问题。\n您将在下面找到相关的系统文档片段。")
    .dynamic_context(1, vector_store)
    .build();

  // 提示并打印响应
  let response = rag_agent.prompt("这个协作笔记系统的AI功能有哪些？").await?;

  println!("{}", response);

  Ok(())
}
