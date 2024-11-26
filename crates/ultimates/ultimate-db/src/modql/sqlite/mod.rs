//! Requires the `with-rusqlite` and `with-sea-query` features
//! and provides a very basic `sqlite::FromRow` based on the `Fields` derivation.
//!

pub use ultimate_db_macros::SqliteFromRow;
pub use ultimate_db_macros::SqliteFromValue;
pub use ultimate_db_macros::SqliteToValue;

// -- deprecated
pub use ultimate_db_macros::FromSqliteRow;
pub use ultimate_db_macros::FromSqliteValue;
pub use ultimate_db_macros::ToSqliteValue;

#[deprecated(note = "use SqliteFromRow")]
pub trait FromSqliteRow: SqliteFromRow
where
  Self: Sized,
{
}

pub trait SqliteFromRow
where
  Self: Sized,
{
  #[deprecated(note = "use sqlite_from_row")]
  fn from_sqlite_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
    Self::sqlite_from_row(row)
  }

  fn sqlite_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self>;

  fn sqlite_from_row_partial(row: &rusqlite::Row<'_>, prop_names: &[&str]) -> rusqlite::Result<Self>;
}
