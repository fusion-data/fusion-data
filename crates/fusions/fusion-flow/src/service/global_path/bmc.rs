use modelsql::{ModelManager, base::DbBmc, generate_pg_bmc_filter};

use super::{GlobalPath, GlobalPathFilter};

pub struct GlobalPathBmc;
impl DbBmc for GlobalPathBmc {
  const TABLE: &'static str = "global_path";

  fn has_creation_timestamps() -> bool {
    false
  }

  fn has_modification_timestamps() -> bool {
    false
  }
}
generate_pg_bmc_filter!(
  Bmc: GlobalPathBmc,
  Entity: GlobalPath,
  Filter: GlobalPathFilter,
);

impl GlobalPathBmc {
  pub async fn obtain_lock(
    mm: &ModelManager,
    path: &str,
    value: Option<String>,
    revision: Option<i64>,
  ) -> modelsql::Result<bool> {
    let sql_str = if revision.is_some() {
      format!(
        r#"INSERT INTO {}(path, value)
           VALUES ($1, $2)
           ON CONFLICT(path)
           DO UPDATE SET value    = excluded.value,
                         revision = {}.revision + 1
           WHERE {}.revision = $3;"#,
        Self::TABLE,
        Self::TABLE,
        Self::TABLE
      )
    } else {
      format!(
        r#"INSERT INTO {}(path, value)
           VALUES ($1, $2)
           ON CONFLICT(path)
           DO UPDATE SET value = excluded.value;"#,
        Self::TABLE
      )
    };

    let mut query = sqlx::query(&sql_str).bind(path).bind(value);
    if let Some(version) = revision {
      query = query.bind(version);
    }

    let ret = mm.dbx().use_postgres(async |dbx| dbx.execute(query).await).await?;
    Ok(ret == 1)
  }
}
