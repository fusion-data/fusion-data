use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
  // Create a connection pool
  //  for MySQL/MariaDB, use MySqlPoolOptions::new()
  //  for SQLite, use SqlitePoolOptions::new()
  //  etc.
  let pool: Pool<Postgres> =
    PgPoolOptions::new().max_connections(5).connect("postgres://postgres:password@localhost/test").await?;

  // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL/MariaDB)
  let row: (i64,) = sqlx::query_as("SELECT $1").bind(150_i64).fetch_one(&pool).await?;

  assert_eq!(row.0, 150);

  Ok(())
}
