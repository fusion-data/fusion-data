pub mod base;
mod config;
mod error;
pub mod field;
pub mod filter;
pub mod id;
pub mod includes;
mod model_manager;
pub mod page;
mod sea_utils;
pub mod store;
pub mod utils;

pub use config::DbConfig;
pub use error::{Result, SqlError};
use field::HasSeaFields;
pub use model_manager::ModelManager;
pub use sea_utils::*;
use sqlx::{postgres::PgRow, FromRow};

pub trait DbRowType: HasSeaFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}
