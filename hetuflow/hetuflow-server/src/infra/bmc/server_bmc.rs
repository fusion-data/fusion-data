use fusion_common::time::now_offset;
use fusionsql::filter::OrderBys;
use fusionsql::{
  ModelManager, SqlError, base::DbBmc, filter::OpValInt32, generate_pg_bmc_common, generate_pg_bmc_filter,
};

use hetuflow_core::models::{SchedServer, ServerFilter, ServerForRegister, ServerForUpdate};
use hetuflow_core::types::ServerStatus;

/// ServerBmc 实现
pub struct ServerBmc;

impl DbBmc for ServerBmc {
  const TABLE: &str = "sched_server";
  const ID_GENERATED_BY_DB: bool = false;
  fn _has_created_by() -> bool {
    false
  }
  fn _has_updated_at() -> bool {
    false
  }
  fn _has_updated_by() -> bool {
    false
  }
  fn _default_order_bys() -> Option<OrderBys> {
    Some("!id".into())
  }
}

generate_pg_bmc_common!(
  Bmc: ServerBmc,
  Entity: SchedServer,
  ForUpdate: ServerForUpdate,
  ForInsert: ServerForRegister,
);

generate_pg_bmc_filter!(
  Bmc: ServerBmc,
  Entity: SchedServer,
  Filter: ServerFilter,
);

impl ServerBmc {
  /// 查找活跃的服务器
  pub async fn find_active_servers(mm: &ModelManager) -> Result<Vec<SchedServer>, SqlError> {
    let filter = ServerFilter { status: Some(OpValInt32::eq(ServerStatus::Active as i32)), ..Default::default() };

    Self::find_many(mm, vec![filter], None).await
  }

  /// 查找领导者服务器
  pub async fn find_leader_server(mm: &ModelManager) -> Result<Option<SchedServer>, SqlError> {
    let filter = ServerFilter { status: Some(OpValInt32::eq(ServerStatus::Active as i32)), ..Default::default() };

    let servers = Self::find_many(mm, vec![filter], None).await?;
    Ok(servers.into_iter().next())
  }

  pub async fn register(mm: &ModelManager, server: ServerForRegister) -> Result<(), SqlError> {
    let sql = r#"insert into sched_server(id, name, address, status, last_heartbeat_at)
      values ($1, $2, $3, $4, $5)
      on conflict (id) do update set name           = excluded.name,
                                    address         = excluded.address,
                                    status          = excluded.status,
                                    last_heartbeat_at = excluded.last_heartbeat_at"#;
    let db = mm.dbx().db_postgres()?;
    let query = sqlx::query(sql)
      .bind(&server.id)
      .bind(&server.name)
      .bind(&server.address)
      .bind(server.status as i32)
      .bind(now_offset());
    let rows_affected = db.execute(query).await?;
    if rows_affected == 1 {
      Ok(())
    } else {
      Err(SqlError::ExecuteError {
        table: Self::qualified_table_name(),
        message: format!("Register server error, id: {}, name: {}", server.id, server.name),
      })
    }
  }

  /// 更新服务器的 namespace_id 绑定
  pub async fn update_server_namespace_bind(
    mm: &ModelManager,
    server_id: &str,
    bind_namespaces: Vec<String>,
  ) -> Result<(), SqlError> {
    let entity_u = ServerForUpdate { bind_namespaces: Some(bind_namespaces), ..Default::default() };
    Self::update_by_id(mm, server_id, entity_u).await
  }

  pub async fn count_namespace_by_server(mm: &ModelManager, server_id: &str) -> Result<u32, SqlError> {
    let sql = "select sum(coalesce(array_length(bind_namespaces, 1), 0)) from sched_server where id = $1";
    let db = mm.dbx().db_postgres()?;
    let query = sqlx::query_as::<_, (i64,)>(sql).bind(server_id);
    let (count,) = db.fetch_one(query).await?;
    Ok(count as u32)
  }
}
