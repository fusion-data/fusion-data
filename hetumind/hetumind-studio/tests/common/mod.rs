//! tests/common/mod.rs
#![allow(dead_code)]

use axum::Router;
use axum_test::TestServer;
use config::File;
use fusion_core::{DataError, application::Application};
use fusion_db::DbPlugin;
use fusionsql::{ModelManager, store::DbxPostgres};
use hetumind_core::workflow::{ErrorHandlingStrategy, ExecutionMode, WorkflowId, WorkflowStatus};
use hetumind_studio::{
  endpoint,
  infra::{db::execution::ExecutionStorePlugin, queue::QueueProviderPlugin},
  runtime::workflow::WorkflowEnginePlugin,
  utils::NodeRegistryPlugin,
};
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::{Connection, Executor, PgConnection};
use tokio::sync::OnceCell;

// 使用 std::sync::Once 确保只初始化一次
static ONCE: Lazy<OnceCell<Application>> = Lazy::new(OnceCell::new);

pub async fn get_server() -> TestServer {
  let context = TestContext::setup().await;

  // 清理测试数据
  cleanup_test_data(&context.dbx).await;

  let mut server = TestServer::new(context.router).unwrap();
  server.add_header("Authorization", format!("Bearer {}", ADMIN_TOKEN));
  server
}

// 清理测试数据的函数
async fn cleanup_test_data(dbx: &DbxPostgres) {
  let mut conn = dbx.db().acquire().await.unwrap();

  // 按照外键依赖顺序删除数据
  conn.execute("DELETE FROM execution_data").await.ok();
  conn.execute("DELETE FROM execution_entity").await.ok();
  conn.execute("DELETE FROM workflow_entity").await.ok();
}

// The test context that will be shared between tests.
pub struct TestContext {
  pub router: Router,
  pub dbx: DbxPostgres,
}

impl TestContext {
  pub async fn setup() -> Self {
    // 使用 std::sync::Once 确保只运行一次全局设置
    ONCE.get_or_init(|| async { init_application().await.unwrap() }).await;

    let application = Application::global();
    let mm = application.component::<ModelManager>();

    // 运行数据库迁移
    mm.dbx()
      .use_postgres(|dbx| async move {
        sqlx::migrate::Migrator::new(std::path::Path::new("scripts/migrations"))
          .await
          .unwrap()
          .run(dbx.db())
          .await
          .unwrap();
        Ok(())
      })
      .await
      .unwrap();

    let router = endpoint::api::routes().with_state(application);
    let dbx = mm.dbx().db_postgres().unwrap().clone();

    TestContext { router, dbx }
  }
}

async fn init_application() -> Result<Application, DataError> {
  Application::builder()
    .add_config_source(File::with_name("resources/test-app.toml"))
    .add_plugin(DbPlugin) // ModelManager
    .add_plugin(NodeRegistryPlugin) // NodeRegistry
    .add_plugin(ExecutionStorePlugin) // ExecutionStoreService
    .add_plugin(QueueProviderPlugin) // QueueProvider
    .add_plugin(WorkflowEnginePlugin) // WorkflowEngineService
    .build()
    .await
}

async fn create_test_database() {
  // We need to set up the database for the tests.
  // We will create a new database for each test run.
  let db_url = "postgresql://fusiondata:2025.Fusiondata@localhost:5432/template1";
  let mut conn = PgConnection::connect(db_url).await.unwrap();
  conn.execute("drop database if exists fusiondata_test;").await.unwrap();
  conn.execute("create database fusiondata_test owner=fusiondata;").await.unwrap();
}

pub fn create_test_workflow_json() -> (WorkflowId, serde_json::Value) {
  let id = WorkflowId::now_v7();
  let workflow_json = json!({
    "id": id,
    "name": "My Test Workflow",
    "status": WorkflowStatus::Draft,
    "settings": {
      "execution_timeout": 10 * 60,
      "error_handling": ErrorHandlingStrategy::StopOnFirstError,
      "execution_mode": ExecutionMode::Local,
      "remark": "Test Workflow",
    },
    "meta": {},
    "nodes": [{
      "id": "start",
      "kind": {
        "name": "Start",
        "version": 1,
      },
      "position": {"x": 100, "y": 100},
      "properties": {}
    }],
    "connections": [],
    "pin_data": {"data": {}},
  });
  (id, workflow_json)
}

pub const ADMIN_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..DgILOqv5oLrhjyZ_czY3SQ.0VtJ2ukXRcZ9XiJzB0rF99q2KM-AmZPe1HeFOHlHcwo.1rFXa0FUUBQX2y5xaTcm7A";
