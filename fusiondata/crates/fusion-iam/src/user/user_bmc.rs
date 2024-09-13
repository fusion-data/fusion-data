use ultimate_db::{base::DbBmc, generate_common_bmc_fns, generate_filter_bmc_fns};

use super::{User, UserFilter, UserForCreate, UserForUpdate};

pub struct UserBmc;
impl DbBmc for UserBmc {
  const SCHEMA: &'static str = "iam";
  const TABLE: &'static str = "user";
}

generate_common_bmc_fns!(
  Bmc: UserBmc,
  Entity: User,
  ForCreate: UserForCreate,
  ForUpdate: UserForUpdate,
);

generate_filter_bmc_fns!(
  Bmc: UserBmc,
  Entity: User,
  Filter: UserFilter,
);
