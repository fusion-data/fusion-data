use ultimate_db::{base::DbBmc, generate_filter_bmc_fns, ModelManager};

use super::{GlobalPath, GlobalPathFilter};

pub struct GlobalPathBmc;
impl DbBmc for GlobalPathBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "global_path";

  fn has_creation_timestamps() -> bool {
    false
  }

  fn has_modification_timestamps() -> bool {
    false
  }
}
generate_filter_bmc_fns!(
  Bmc: GlobalPathBmc,
  Entity: GlobalPath,
  Filter: GlobalPathFilter,
);

impl GlobalPathBmc {
  pub async fn obtain_lock(
    mm: &ModelManager,
    path: &str,
    value: Option<String>,
    version: Option<i64>,
  ) -> ultimate_db::Result<bool> {
    let sql_str = if version.is_some() {
      format!(
        r#"INSERT INTO {}.{}(path, value)
           VALUES (?, ?)
           ON CONFLICT(path)
           DO UPDATE SET value    = excluded.value,
                         revision = {}.revision + 1
           WHERE {}.revision = ?;"#,
        Self::SCHEMA,
        Self::TABLE,
        Self::TABLE,
        Self::TABLE
      )
    } else {
      format!(
        r#"INSERT INTO {}.{}(path, value)
           VALUES (?, ?)
           ON CONFLICT(path)
           DO UPDATE SET value = excluded.value;"#,
        Self::SCHEMA,
        Self::TABLE
      )
    };

    let mut query = sqlx::query(&sql_str).bind(value).bind(path);
    if let Some(version) = version {
      query = query.bind(version);
    }

    let ret = mm.dbx().execute(query).await?;
    Ok(ret == 1)
  }
}
