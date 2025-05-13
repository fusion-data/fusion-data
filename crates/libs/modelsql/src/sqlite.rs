use crate::field::HasSeaFields;
use sqlx::sqlite::SqliteRow;
use sqlx::FromRow;

pub trait SqliteRowType: HasSeaFields + for<'r> FromRow<'r, SqliteRow> + Unpin + Send {}
