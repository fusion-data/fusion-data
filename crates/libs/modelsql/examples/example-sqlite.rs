use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use log::info;
use modelsql::base::DbBmc;
use modelsql::store::Dbx;
use modelsql::{DbConfig, ModelManager, generate_sqlite_bmc_common, generate_sqlite_bmc_filter};
use modelsql::{
  field::{FieldMask, Fields},
  filter::Page,
  page::PageResult,
  sqlite::SqliteRowType,
};
use modelsql_macros::FilterNodes;
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use std::env;
use std::time::Duration;
use ultimate_common::ctx::Ctx;

#[derive(Debug, Serialize, Deserialize, FromRow, Fields)]
#[enum_def]
pub struct User {
  pub id: i64,
  pub name: String,
  pub status: i32,
  pub created_at: DateTime<Utc>,
  pub created_by: i64,
}
impl SqliteRowType for User {}

#[derive(Debug, Clone, FilterNodes)]
pub struct UserFilter {
  pub name: Option<String>,
  pub status: Option<i32>,
}

#[derive(Fields)]
pub struct UserForCreate {
  pub name: String,
  pub status: i32,
}

#[derive(Fields)]
pub struct UserForUpdate {
  pub name: Option<String>,
  pub status: Option<i32>,
  pub update_mask: FieldMask,
}

pub struct UserBmc;
impl DbBmc for UserBmc {
  const TABLE: &'static str = "user";
}

generate_sqlite_bmc_common!(Bmc: UserBmc, Entity: User, ForCreate: UserForCreate, ForUpdate: UserForUpdate,);
generate_sqlite_bmc_filter!(Bmc: UserBmc, Entity: User, Filter: UserFilter,);

#[derive(Debug)]
struct SqliteModel {
  db: ModelManager,
}

impl SqliteModel {
  async fn new(db_path: &str) -> Result<Self> {
    let db_config: DbConfig = serde_json::from_value(json!({
      "enable": true,
      "url": db_path,
      "max_connections": 5,
      "min_connections": 1,
      "acquire_timeout": Duration::from_secs(10),
    }))?;

    let db = ModelManager::new(&db_config, Some("example-sqlite"))
      .await
      .unwrap()
      .with_ctx(Ctx::new_super_admin());

    // 创建表
    match db.dbx() {
      Dbx::Sqlite(dbx) => {
        sqlx::query(
          r#"
          CREATE TABLE IF NOT EXISTS user (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              name TEXT NOT NULL,
              status INT NOT NULL,
              created_at DATETIME NOT NULL,
              created_by BIGINT NOT NULL
          )
          "#,
        )
        .execute(dbx.db())
        .await?;
      }
      _ => return Err(anyhow!("sqlite")),
    }

    Ok(Self { db })
  }

  async fn create(&self, for_create: UserForCreate) -> Result<i64> {
    UserBmc::create(&self.db, for_create).await.map_err(|e| anyhow!(e))
  }

  async fn get(&self, id: i64) -> Result<User> {
    UserBmc::find_by_id(&self.db, id).await.map_err(|e| anyhow!(e))
  }

  async fn update(&self, id: i64, for_update: UserForUpdate) -> Result<()> {
    UserBmc::update_by_id(&self.db, id, for_update).await.map_err(|e| anyhow!(e))
  }

  async fn delete(&self, id: i64) -> Result<()> {
    UserBmc::delete_by_id(&self.db, id).await.map_err(|e| anyhow!(e))
  }

  async fn list(&self, filter: UserFilter, page: Page) -> Result<PageResult<User>> {
    UserBmc::page(&self.db, vec![filter], page).await.map_err(|e| anyhow!(e))
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  // 初始化日志
  logforth::stdout().apply();

  // 获取当前目录的绝对路径
  let current_dir = env::current_dir()?;
  let db_path = format!("file:{}?mode=rwc", current_dir.join("var").join("test.db").display());

  println!("Database path: {}", db_path);

  // 初始化用户模型
  let user_model = SqliteModel::new(&db_path).await?;

  // 创建用户
  let user_id = user_model.create(UserForCreate { name: "张三".to_string(), status: 1 }).await?;
  info!("创建用户成功，ID: {}", user_id);

  // 查询用户
  if let Ok(user) = user_model.get(user_id).await {
    info!("查询用户: {:?}", user);
  }

  // 更新用户
  let updated = user_model
    .update(
      user_id,
      UserForUpdate { name: Some("张三(已更新)".to_string()), status: Some(2), update_mask: FieldMask::default() },
    )
    .await
    .ok();
  info!("更新用户结果: {:?}", updated);

  // 再次查询用户
  if let Ok(user) = user_model.get(user_id).await {
    info!("更新后的用户: {:?}", user);
  }

  // 创建更多用户用于分页测试
  user_model.create(UserForCreate { name: "李四".to_string(), status: 1 }).await?;
  user_model.create(UserForCreate { name: "王五".to_string(), status: 2 }).await?;
  user_model.create(UserForCreate { name: "赵六".to_string(), status: 1 }).await?;

  // 分页查询
  let page_result = user_model
    .list(
      UserFilter { name: Some("李四".to_string()), status: Some(1) },
      Page { limit: Some(10), page: Some(1), ..Default::default() },
    )
    .await?;
  info!("分页查询结果: 总数 {}, 当前页数据: {:?}", page_result.page.total, page_result.result);

  // 删除用户
  let deleted = user_model.delete(user_id).await.ok();
  info!("删除用户结果: {:?}", deleted);

  Ok(())
}
