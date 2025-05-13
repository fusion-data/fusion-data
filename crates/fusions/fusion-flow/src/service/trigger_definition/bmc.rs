use chrono::Utc;
use modelsql::field::HasSeaFields;
use modelsql::{ModelManager, Result, base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};
use sea_query::{Expr, PostgresQueryBuilder, Query, all, any};
use sea_query_binder::SqlxBinder;

use crate::service::sched_namespace::SchedNamespaceIden;
use crate::service::trigger_definition::TriggerDefinitionIden;

use super::{TriggerDefinition, TriggerDefinitionFilter, TriggerDefinitionForCreate, TriggerDefinitionForUpdate};

pub struct TriggerDefinitionBmc;
impl DbBmc for TriggerDefinitionBmc {
  const TABLE: &'static str = "trigger_definition";
}

generate_pg_bmc_common!(
  Bmc: TriggerDefinitionBmc,
  Entity: TriggerDefinition,
  ForCreate: TriggerDefinitionForCreate,
  ForUpdate: TriggerDefinitionForUpdate,
);

generate_pg_bmc_filter!(
  Bmc: TriggerDefinitionBmc,
  Entity: TriggerDefinition,
  Filter: TriggerDefinitionFilter,
);

impl TriggerDefinitionBmc {
  pub async fn scan_next_triggers(mm: &ModelManager, node_id: &str) -> Result<Vec<TriggerDefinition>> {
    let now = Utc::now();
    let mut query = Query::select();
    query
      .from(Self::table_ref())
      .columns(TriggerDefinition::sea_column_refs_with_rel(TriggerDefinitionIden::Table))
      .inner_join(
        SchedNamespaceIden::Table,
        Expr::col((TriggerDefinitionIden::Table, TriggerDefinitionIden::NamespaceId))
          .eq(Expr::col((SchedNamespaceIden::Table, SchedNamespaceIden::Id))),
      )
      .cond_where(all![
        Expr::col((SchedNamespaceIden::Table, SchedNamespaceIden::NodeId)).eq(node_id),
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

    let list = mm
      .dbx()
      .use_postgres(async |dbx| {
        let items = dbx.fetch_all(sqlx_query).await?;
        Ok(items)
      })
      .await?;
    Ok(list)
  }
}
