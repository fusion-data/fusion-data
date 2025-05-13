use crate::field::HasSeaFields;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

pub trait PgRowType: HasSeaFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}
