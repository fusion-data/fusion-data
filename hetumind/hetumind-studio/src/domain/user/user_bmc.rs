use std::sync::OnceLock;

use fusionsql::{
  ModelManager, SqlError,
  base::{BmcConfig, DbBmc},
  filter::{FilterGroups, FilterNode, OpValInt64},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};
use sea_query::{Condition, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;

use crate::domain::user::UserEntityIden;

use super::{UserEntity, UserFilter, UserForCreate, UserForUpdate};

pub struct UserBmc;

impl DbBmc for UserBmc {
  fn _bmc_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table("user_entity").with_id_generated_by_db(true))
  }
}
generate_pg_bmc_common!(
  Bmc: UserBmc,
  Entity: UserEntity,
  ForCreate: UserForCreate,
  ForUpdate: UserForUpdate,
);
generate_pg_bmc_filter!(
  Bmc: UserBmc,
  Entity: UserEntity,
  Filter: UserFilter,
);
impl UserBmc {
  pub async fn set_new_password(mm: &ModelManager, id: i64, pwd_hash: String) -> Result<(), SqlError> {
    let mut query = Query::update();
    let fields = vec![(UserEntityIden::Password, pwd_hash.into())];
    query.table(Self::_bmc_config().table_ref()).values(fields);
    let filters: FilterGroups = FilterNode::new("id", OpValInt64::eq(id)).into();
    let cond: Condition = filters.try_into()?;
    query.cond_where(cond);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    mm.dbx()
      .use_postgres(|dbx| async move {
        let sqlx_query = sqlx::query_with(&sql, values);
        dbx.execute(sqlx_query).await?;
        Ok(())
      })
      .await?;
    Ok(())
  }
}
