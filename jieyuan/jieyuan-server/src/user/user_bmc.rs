use std::sync::OnceLock;

use fusionsql::{
  ModelManager, SqlError,
  base::{BmcConfig, DbBmc, pg_page},
  generate_pg_bmc_common, generate_pg_bmc_filter,
};

use jieyuan_core::model::{
  TABLE_USER, User, UserChangeQueryReq, UserChangeQueryResp, UserFilter, UserForCreate, UserForUpdate,
};

pub struct UserBmc;
impl DbBmc for UserBmc {
  fn _static_config() -> &'static BmcConfig {
    static CONFIG: OnceLock<BmcConfig> = OnceLock::new();
    CONFIG.get_or_init(|| BmcConfig::new_table(TABLE_USER).with_use_logical_deletion(true))
  }
}

generate_pg_bmc_common!(
  Bmc: UserBmc,
  Entity: User,
  ForCreate: UserForCreate,
  ForUpdate: UserForUpdate,
);

generate_pg_bmc_filter!(
  Bmc: UserBmc,
  Entity: User,
  Filter: UserFilter,
);

impl UserBmc {
  pub async fn query_user_changes(mm: &ModelManager, req: UserChangeQueryReq) -> Result<UserChangeQueryResp, SqlError> {
    let paged = pg_page::<Self, _, _>(mm, req.filters, req.page).await?;
    Ok(UserChangeQueryResp { page: paged.page, result: paged.result })
  }
}
