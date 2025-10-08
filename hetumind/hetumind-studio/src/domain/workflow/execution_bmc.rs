use hetumind_core::workflow::{ExecutionFilter, ExecutionForUpdate};
use fusionsql::{base::DbBmc, generate_pg_bmc_common, generate_pg_bmc_filter};

use crate::domain::workflow::ExecutionDataEntity;

use super::ExecutionEntity;

pub struct ExecutionBmc;
impl DbBmc for ExecutionBmc {
  const TABLE: &'static str = "execution_entity";
}
generate_pg_bmc_common!(
  Bmc: ExecutionBmc,
  Entity: ExecutionEntity,
  ForUpdate: ExecutionForUpdate,
  ForInsert: ExecutionEntity,
);
generate_pg_bmc_filter!(
  Bmc: ExecutionBmc,
  Entity: ExecutionEntity,
  Filter: ExecutionFilter,
  ForUpdate: ExecutionForUpdate,
);

pub struct ExecutionDataBmc;
impl DbBmc for ExecutionDataBmc {
  const TABLE: &'static str = "execution_data";
}
generate_pg_bmc_common!(
  Bmc: ExecutionDataBmc,
  Entity: ExecutionDataEntity,
  ForInsert: ExecutionDataEntity,
);
