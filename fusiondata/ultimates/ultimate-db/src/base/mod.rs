use sea_query::Iden;

mod crud_fns;
mod db_bmc;
mod macro_utils;
mod utils;

pub use crud_fns::*;
pub use db_bmc::*;
pub use utils::*;

const LIST_LIMIT_DEFAULT: i64 = 1000;
const LIST_LIMIT_MAX: i64 = 5000;

#[derive(Iden)]
pub enum CommonIden {
  Id,
  OwnerId,
  LogiscalDeletion,
  OptimisticLock,
}

#[derive(Iden)]
pub enum TimestampIden {
  Cid,
  Ctime,
  Mid,
  Mtime,
}
