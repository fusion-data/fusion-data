use crate::field::HasSeaFields;
use sqlx::FromRow;
use sqlx::sqlite::SqliteRow;

pub trait SqliteRowType: HasSeaFields + for<'r> FromRow<'r, SqliteRow> + Unpin + Send {}
