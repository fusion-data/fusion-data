use fusionsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use jieyuan_core::model::{TABLE_USER, User, UserFilter, UserForCreate, UserForUpdate};

pub struct UserBmc;
impl DbBmc for UserBmc {
  const TABLE: &'static str = TABLE_USER;
  fn _use_logical_deletion() -> bool {
    true
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
