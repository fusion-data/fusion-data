use fusion_common::time::UtcDateTime;
use modelsql::{
  ModelManager, SqlError, base::DbBmc, filter::OpValsInt32, generate_pg_bmc_common, generate_pg_bmc_filter,
};
use uuid::Uuid;

use hetuflow_core::models::{ServerEntity, ServerFilter, ServerForRegister, ServerForUpdate};
use hetuflow_core::types::ServerStatus;

/// ServerBmc 实现
pub struct ServerBmc;

impl DbBmc for ServerBmc {
  const TABLE: &str = "sched_server";
}

generate_pg_bmc_common!(
  Bmc: ServerBmc,
  Entity: ServerEntity,
  ForUpdate: ServerForUpdate,
  ForInsert: ServerForRegister,
);

generate_pg_bmc_filter!(
  Bmc: ServerBmc,
  Entity: ServerEntity,
  Filter: ServerFilter,
);

impl ServerBmc {
  /// 查找活跃的服务器
  pub async fn find_active_servers(mm: &ModelManager) -> Result<Vec<ServerEntity>, SqlError> {
    let filter = ServerFilter { status: Some(OpValsInt32::eq(ServerStatus::Active as i32)), ..Default::default() };

    Self::find_many(mm, vec![filter], None).await
  }

  /// 查找领导者服务器
  pub async fn find_leader_server(mm: &ModelManager) -> Result<Option<ServerEntity>, SqlError> {
    let filter = ServerFilter { status: Some(OpValsInt32::eq(ServerStatus::Active as i32)), ..Default::default() };

    let servers = Self::find_many(mm, vec![filter], None).await?;
    Ok(servers.into_iter().next())
  }

  pub async fn register(mm: &ModelManager, server: ServerForRegister) -> Result<(), SqlError> {
    let sql = r#"insert into sched_server(id, name, address, status, created_by, created_at)
      values ($1, $2, $3, $4, $5, $6)
      on conflict (id) do update set name           = excluded.name,
                                    address         = excluded.address,
                                    status          = excluded.status,
                                    updated_by      = excluded.created_by,
                                    updated_at      = excluded.created_at"#;
    let db = mm.dbx().db_postgres()?;
    let ctx = mm.ctx_ref()?;
    let query = sqlx::query(sql)
      .bind(server.id)
      .bind(server.name.clone())
      .bind(server.address)
      .bind(server.status as i32)
      .bind(ctx.uid())
      .bind(UtcDateTime::from(*ctx.req_time()));
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
    server_id: Uuid,
    bind_namespaces: Vec<Uuid>,
  ) -> Result<(), SqlError> {
    let entity_u = ServerForUpdate { bind_namespaces: Some(bind_namespaces), ..Default::default() };
    Self::update_by_id(mm, server_id, entity_u).await
  }
}
