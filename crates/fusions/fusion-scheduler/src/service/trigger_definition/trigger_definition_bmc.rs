use chrono::Utc;
use modql::field::HasSeaFields;
use sea_query::{all, any, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use ultimate_db::{base::DbBmc, generate_common_bmc_fns, generate_filter_bmc_fns, ModelManager, Result};

use crate::service::{
  sched_node::{SchedNodeBmc, SchedNodeIden},
  trigger_definition::TriggerDefinitionIden,
};

use super::{TriggerDefinition, TriggerDefinitionFilter, TriggerDefinitionForCreate, TriggerDefinitionForUpdate};

pub struct TriggerDefinitionBmc;
impl DbBmc for TriggerDefinitionBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "trigger_definition";
}

generate_common_bmc_fns!(
  Bmc: TriggerDefinitionBmc,
  Entity: TriggerDefinition,
  ForCreate: TriggerDefinitionForCreate,
  ForUpdate: TriggerDefinitionForUpdate,
);

generate_filter_bmc_fns!(
  Bmc: TriggerDefinitionBmc,
  Entity: TriggerDefinition,
  Filter: TriggerDefinitionFilter,
);

impl TriggerDefinitionBmc {
  pub async fn scan_next_triggers(mm: &ModelManager, node_id: i64) -> Result<Vec<TriggerDefinition>> {
    let now = Utc::now();
    let mut query = Query::select();
    query
      .from(Self::table_ref())
      .columns(TriggerDefinition::sea_column_refs_with_rel(TriggerDefinitionIden::Table))
      .inner_join(
        SchedNodeBmc::table_ref(),
        Expr::col((SchedNodeIden::Table, SchedNodeIden::Id))
          .eq(Expr::col((TriggerDefinitionIden::Table, TriggerDefinitionIden::NamespaceId))),
      )
      .cond_where(all![
        Expr::col((SchedNodeIden::Table, SchedNodeIden::Id)).eq(node_id),
        Expr::col((TriggerDefinitionIden::Table, TriggerDefinitionIden::Status)).eq(100),
        Expr::col((TriggerDefinitionIden::Table, TriggerDefinitionIden::RefreshOccurrence)).lte(now),
        any![
          Expr::col((TriggerDefinitionIden::Table, TriggerDefinitionIden::ValidTime)).is_null(),
          Expr::col((TriggerDefinitionIden::Table, TriggerDefinitionIden::ValidTime)).lte(now)
        ],
        any![
          Expr::col((TriggerDefinitionIden::Table, TriggerDefinitionIden::InvalidTime)).is_null(),
          Expr::col((TriggerDefinitionIden::Table, TriggerDefinitionIden::InvalidTime)).gte(now)
        ]
      ]);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let sqlx_query = sqlx::query_as_with::<_, TriggerDefinition, _>(&sql, values);
    let list = mm.dbx().fetch_all(sqlx_query).await?;
    Ok(list)
  }
}
