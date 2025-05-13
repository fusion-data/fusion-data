use modelsql::{
  base::{self, prep_fields_for_update, CommonIden, DbBmc},
  field::HasSeaFields,
  generate_pg_bmc_common, generate_pg_bmc_filter, ModelManager, Result, SqlError,
};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use ultimate_common::time::{now, UtcDateTime};

use super::{SchedNode, SchedNodeFilter, SchedNodeForCreate, SchedNodeForUpdate};

pub struct SchedNodeBmc;
impl DbBmc for SchedNodeBmc {
  const TABLE: &'static str = "sched_node";
}

impl SchedNodeBmc {
  pub async fn find_active_master(mm: &ModelManager, valid_check_time: UtcDateTime) -> Result<Option<SchedNode>> {
    let sql = format!(
      r#"SELECT sn.* FROM {} sn
         INNER JOIN global_path g ON sn.id = g.value
         WHERE sn.last_check_time > $1"#,
      Self::TABLE
    );
    let query = sqlx::query_as::<_, SchedNode>(&sql).bind(valid_check_time);

    let node = mm
      .dbx()
      .use_postgres(async |dbx| {
        let node = dbx.fetch_optional(query).await?;
        Ok(node)
      })
      .await?;
    Ok(node)
  }

  pub async fn update_and_return(mm: &ModelManager, id: &str, data: SchedNodeForUpdate) -> Result<SchedNode> {
    let ctx = mm.ctx_ref()?;

    // -- Prep Fields
    let mut fields = data.not_none_sea_fields();
    if Self::has_modification_timestamps() {
      fields = prep_fields_for_update::<Self>(fields, ctx);
    }

    // -- Build query
    let fields = fields.for_sea_update();
    let mut query = Query::update();
    query
      .table(Self::table_ref())
      .values(fields)
      .and_where(Expr::col(CommonIden::Id).eq(id))
      .returning_all();

    // -- Execute query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, SchedNode, _>(&sql, values);
    let node = mm
      .dbx()
      .use_postgres(async |dbx| {
        let node = dbx.fetch_optional(sqlx_query).await?;
        Ok(node)
      })
      .await?
      .ok_or_else(|| SqlError::EntityNotFound { schema: Self::SCHEMA, entity: Self::TABLE, id: id.into() })?;
    Ok(node)
  }

  pub async fn update(mm: &ModelManager, vec: Vec<SchedNodeFilter>, entity_u: SchedNodeForUpdate) -> Result<u64> {
    base::update::<Self, _, _>(mm, vec, entity_u).await
  }

  pub(crate) async fn register(mm: &ModelManager, entity_c: SchedNodeForCreate) -> Result<()> {
    let sql = format!(
      r#"insert into {}(id, kind, addr, status, last_check_time, cid, ctime)
      values ($1, $2, $3, $4, $5, $6, $7)
      on conflict (id)
          do update set last_check_time = excluded.last_check_time,
                        mid             = $6,
                        mtime           = $7;"#,
      Self::TABLE
    );
    let query = sqlx::query(&sql)
      .bind(entity_c.id)
      .bind(entity_c.kind)
      .bind(entity_c.addr)
      .bind(entity_c.status)
      .bind(entity_c.last_check_time)
      .bind(mm.ctx_ref()?.uid())
      .bind(now());

    mm.dbx()
      .use_postgres(async |dbx| {
        dbx.execute(query).await?;
        Ok(())
      })
      .await?;
    Ok(())
  }
}

generate_pg_bmc_common!(
  Bmc: SchedNodeBmc,
  Entity: SchedNode,
  ForCreate: SchedNodeForCreate,
  ForUpdate: SchedNodeForUpdate,
);

generate_pg_bmc_filter!(
  Bmc: SchedNodeBmc,
  Entity: SchedNode,
  Filter: SchedNodeFilter,
);
