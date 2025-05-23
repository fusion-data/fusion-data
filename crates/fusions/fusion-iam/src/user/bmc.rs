use modelsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use super::{User, UserFilter, UserForCreate, UserForUpdate};

pub struct UserBmc;
impl DbBmc for UserBmc {
  const TABLE: &'static str = "user";
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
