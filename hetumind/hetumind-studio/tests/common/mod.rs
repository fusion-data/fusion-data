//! tests/common/mod.rs

use axum::Router;
use axum_test::TestServer;
use config::File;
use fusion_core::application::Application;
use fusionsql::{ModelManager, store::DbxPostgres};
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::Executor;
use tokio::sync::OnceCell;

use hetumind_core::workflow::{ErrorHandlingStrategy, ExecutionMode, WorkflowId, WorkflowStatus};
use hetumind_studio::{endpoint, start::app_builder};

// 确保只初始化一次
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
    ONCE
      .get_or_init(|| async { app_builder(Some(File::with_name("resources/test-app.toml"))).run().await.unwrap() })
      .await;

    let application = Application::global();
    let mm = application.component::<ModelManager>();

    let router = endpoint::api::routes().with_state(application);
    let dbx = mm.dbx().db_postgres().unwrap().clone();

    TestContext { router, dbx }
  }
}

#[allow(dead_code)]
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

pub const ADMIN_TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..03gYECTkpz9mT4yBeslZkw.hzfsYCVvvJYIC8JqQxu3w6MI2puqekcMJ0C6Q0G3FJ9lW9nCaRmUx8im7DGx8Zki.wdKF4I1iMTCFggcA7qjmug";
