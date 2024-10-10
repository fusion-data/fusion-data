use ultimate_db::{base::DbBmc, ModelManager};

use super::LockKind;

pub struct SchedLockBmc;
impl DbBmc for SchedLockBmc {
  const SCHEMA: &'static str = "sched";
  const TABLE: &'static str = "sched_lock";

  fn has_creation_timestamps() -> bool {
    false
  }

  fn has_modification_timestamps() -> bool {
    false
  }
}

impl SchedLockBmc {
  pub async fn obtain_lock(mm: &ModelManager, node_id: &str, lock_kind: LockKind) -> ultimate::Result<bool> {
    let update_for_lock =
      format!("UPDATE {}.sched_lock SET lock_kind = ? WHERE node_id = ? AND lock_kind = ?", Self::SCHEMA);
    let query_u = sqlx::query(&update_for_lock).bind(lock_kind).bind(node_id).bind(lock_kind);
    let mut ret = mm.dbx().execute(query_u).await?;
    if ret == 0 {
      let insert_lock = format!("INSERT INTO {}.sched_lock (node_id, lock_kind) VALUES (?, ?)", Self::SCHEMA);
      let query_i = sqlx::query(&insert_lock).bind(node_id).bind(lock_kind);
      ret = mm.dbx().execute(query_i).await?;
    }

    Ok(ret == 1)
  }
}
