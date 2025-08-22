use modelsql::store::DbxError;
use modelsql::{
  ModelManager, SqlError, base::DbBmc, field::FieldMask, filter::OpValsInt32, generate_pg_bmc_common,
  generate_pg_bmc_filter,
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
    let sql = r#"insert into sched_server(id, name, address, status, description, created_by, created_at)
      values ($1, $2, $3, $4, $5, $6, $7)
      on conflict (id) do update set name            = excluded.name,
                                    address         = excluded.address,
                                    status          = excluded.status,
                                    description     = excluded.description,
                                    updated_by      = excluded.created_by,
                                    updated_at      = excluded.created_at"#;
    let db = mm.dbx().db_postgres()?;
    let ctx = mm.ctx_ref()?;
    let ret = sqlx::query(sql)
      .bind(server.id)
      .bind(server.name.clone())
      .bind(server.address)
      .bind(server.status as i32)
      .bind(server.description)
      .bind(ctx.uid())
      .bind(ctx.req_time())
      .execute(&db)
      .await
      .map_err(DbxError::from)?;
    if ret.rows_affected() == 1 {
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
    let sql = r#"update sched_server
                 set bind_namespaces = $1, updated_by = $2, updated_at = $3
                 where id = $4"#;
    let db = mm.dbx().db_postgres()?;
    let ctx = mm.ctx_ref()?;

    let ret = sqlx::query(sql)
      .bind(&bind_namespaces)
      .bind(ctx.uid())
      .bind(ctx.req_time())
      .bind(server_id)
      .execute(&db)
      .await
      .map_err(DbxError::from)?;

    if ret.rows_affected() == 1 {
      Ok(())
    } else {
      Err(SqlError::ExecuteError {
        table: Self::qualified_table_name(),
        message: format!("Update server namespace_id bind error, server_id: {}", server_id),
      })
    }
  }

  /// 根据服务器 ID 获取绑定的 namespaces
  pub async fn get_server_bind_namespaces(mm: &ModelManager, server_id: Uuid) -> Result<Vec<Uuid>, SqlError> {
    let sql = "select bind_namespaces from sched_server where id = $1";
    let db = mm.dbx().db_postgres()?;

    let row: (Vec<Uuid>,) = sqlx::query_as(sql).bind(server_id).fetch_one(&db).await.map_err(DbxError::from)?;

    Ok(row.0)
  }
}
