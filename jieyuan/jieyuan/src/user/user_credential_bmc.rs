use fusion_core::{DataError, Result};
use fusionsql::{ModelManager, base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use jieyuan_core::model::{
  TABLE_USER_CREDENTIAL, UserCredential, UserCredentialFilter, UserCredentialForInsert, UserCredentialForUpdate,
};

pub struct UserCredentialBmc;
impl DbBmc for UserCredentialBmc {
  const TABLE: &'static str = TABLE_USER_CREDENTIAL;
}

generate_pg_bmc_common!(
  Bmc: UserCredentialBmc,
  Entity: UserCredential,
  ForUpdate: UserCredentialForUpdate,
  ForInsert: UserCredentialForInsert,
);

generate_pg_bmc_filter!(
  Bmc: UserCredentialBmc,
  Entity: UserCredential,
  Filter: UserCredentialFilter,
);

impl UserCredentialBmc {
  /// Get user credential by ID with tenant isolation and row locking
  pub async fn get_by_id_for_update(mm: &ModelManager, user_id: i64, tenant_id: i64) -> Result<Option<UserCredential>> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| DataError::bad_request(format!("Database connection error: {}", e)))?;

    let query = r#"
      SELECT uc.id, uc.encrypted_pwd, uc.token_seq, uc.created_by, uc.created_at, uc.updated_by, uc.updated_at
      FROM iam_user_credential uc
      INNER JOIN iam_tenant_user tu ON uc.id = tu.user_id
      WHERE uc.id = $1 AND tu.tenant_id = $2 AND tu.status = 100
      FOR UPDATE
    "#;

    let mut rows = db
      .fetch_all(sqlx::query_as::<_, UserCredential>(query).bind(user_id).bind(tenant_id))
      .await
      .map_err(|e| DataError::bad_request(format!("Failed to get user credential: {}", e)))?;

    let result = rows.pop();

    Ok(result)
  }

  /// Update password and increment token_seq atomically
  pub async fn update_password_and_bump_token_seq(mm: &ModelManager, user_id: i64, new_hashed_pwd: &str) -> Result<()> {
    let db = mm
      .dbx()
      .db_postgres()
      .map_err(|e| DataError::bad_request(format!("Database connection error: {}", e)))?;

    let query = r#"
      UPDATE iam_user_credential
      SET encrypted_pwd = $2,
          token_seq = token_seq + 1,
          updated_at = now()
      WHERE id = $1
    "#;

    db.execute(sqlx::query(query).bind(user_id).bind(new_hashed_pwd))
      .await
      .map_err(|e| DataError::bad_request(format!("Failed to update password: {}", e)))?;

    Ok(())
  }
}
