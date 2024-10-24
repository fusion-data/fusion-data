use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use ultimate_common::time::UtcDateTime;
use ultimate_db::modql::field::HasSeaFields;
use ultimate_db::{
  base::{self, prep_fields_for_update, CommonIden, DbBmc},
  generate_common_bmc_fns, generate_filter_bmc_fns, ModelManager, Result,
};

use super::{SchedNode, SchedNodeFilter, SchedNodeForCreate, SchedNodeForUpdate};

pub struct SchedNodeBmc;
impl DbBmc for SchedNodeBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "sched_node";
}

impl SchedNodeBmc {
  pub async fn find_active_master(mm: &ModelManager, valid_check_time: UtcDateTime) -> Result<Option<SchedNode>> {
    let sql = format!(
      r#"SELECT sn.* FROM {}.{} sn
         INNER JOIN sched.global_path g ON sn.id = g.value::bigint
         WHERE sn.last_check_time > ?;"#,
      Self::SCHEMA,
      Self::TABLE
    );
    let query = sqlx::query_as::<_, SchedNode>(&sql).bind(valid_check_time);

    let node = mm.dbx().fetch_optional(query).await?;
    Ok(node)
  }

  pub async fn update_and_return(mm: &ModelManager, id: i64, data: SchedNodeForUpdate) -> Result<SchedNode> {
    let ctx = mm.ctx_ref()?;

    // -- Prep Fields
    let mut fields = data.not_none_sea_fields();
    if Self::has_modification_timestamps() {
      fields = prep_fields_for_update::<Self>(fields, ctx);
    }

    // -- Build query
    let fields = fields.for_sea_update();
    let mut query = Query::update();
    query.table(Self::table_ref()).values(fields).and_where(Expr::col(CommonIden::Id).eq(id.clone())).returning_all();

    // -- Execute query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, SchedNode, _>(&sql, values);
    let node = mm.dbx().fetch_one(sqlx_query).await?;
    Ok(node)
  }

  pub async fn update(mm: &ModelManager, vec: Vec<SchedNodeFilter>, entity_u: SchedNodeForUpdate) -> Result<u64> {
    base::update::<Self, _, _>(mm, vec, entity_u).await
  }
}

generate_common_bmc_fns!(
  Bmc: SchedNodeBmc,
  Entity: SchedNode,
  ForCreate: SchedNodeForCreate,
  ForUpdate: SchedNodeForUpdate,
);

generate_filter_bmc_fns!(
  Bmc: SchedNodeBmc,
  Entity: SchedNode,
  Filter: SchedNodeFilter,
);
