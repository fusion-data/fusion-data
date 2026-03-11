use crate::field::HasSeaFields;
use sqlx::FromRow;
use sqlx::postgres::PgRow;

pub trait PgRowType: HasSeaFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}
