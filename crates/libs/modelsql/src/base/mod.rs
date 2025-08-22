use sea_query::Iden;

mod crud_fns;
mod db_bmc;
#[cfg(feature = "with-postgres")]
mod postgres;
#[cfg(feature = "with-sqlite")]
mod sqlite;
mod utils;

pub use crud_fns::*;
pub use db_bmc::*;
#[cfg(feature = "with-postgres")]
pub use postgres::*;
#[cfg(feature = "with-sqlite")]
pub use sqlite::*;
pub use utils::*;

pub const LIST_LIMIT_DEFAULT: u64 = 500;
pub const LIST_LIMIT_MAX: u64 = 5000;

#[derive(Iden)]
pub enum CommonIden {
  // Id,
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
