use sea_query::{Condition, Query, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, postgres::PgRow};

use modelsql_core::filter::{FilterGroups, Page};

use crate::{
  ModelManager, Result, SqlError,
  base::{DbBmc, compute_page, count},
  field::HasSeaFields,
  id::Id,
  page::PageResult,
  store::Dbx,
};

pub async fn pg_find_first<MC, E, F>(mm: &ModelManager, filter: F) -> Result<Option<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
  F: Into<FilterGroups>,
{
  let list = pg_find_many::<MC, E, F>(mm, filter, None).await?;
  Ok(list.into_iter().next())
}

pub async fn pg_find_many<MC, E, F>(mm: &ModelManager, filter: F, page: Option<Page>) -> Result<Vec<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
  F: Into<FilterGroups>,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(E::sea_column_refs());

  // condition from filter
  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  // page
  let page = compute_page::<MC>(page)?;
  page.apply_to_sea_query(&mut query);

  // -- Execute the query
  match mm.dbx() {
    Dbx::Postgres(dbx_postgres) => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
      let entities = dbx_postgres.fetch_all(sqlx_query).await?;
      Ok(entities)
    }
    #[allow(unreachable_patterns)]
    _ => Err(SqlError::InvalidDatabase("Need postgres database")),
  }
}

pub async fn pg_find_many_on<MC, E, F>(mm: &ModelManager, f: F) -> Result<Vec<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
  F: FnOnce(&mut SelectStatement) -> Result<()>,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(E::sea_column_refs());

  // condition from filter and list options
  f(&mut query)?;

  // -- Execute the query
  match mm.dbx() {
    Dbx::Postgres(dbx_postgres) => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
      let entities = dbx_postgres.fetch_all(sqlx_query).await?;
      Ok(entities)
    }
    #[allow(unreachable_patterns)]
    _ => Err(SqlError::InvalidDatabase("Need postgres database")),
  }
}

pub async fn pg_find_by_id<MC, E>(mm: &ModelManager, id: Id) -> Result<E>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
{
  let res = pg_get_by_id::<MC, E>(mm, id.clone()).await?;
  match res {
    Some(entity) => Ok(entity),
    None => Err(SqlError::EntityNotFound { schema: MC::SCHEMA, entity: MC::TABLE, id }),
  }
}

pub async fn pg_get_by_id<MC, E>(mm: &ModelManager, id: Id) -> Result<Option<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(E::sea_column_refs());

  // condition from filter
  let filters: FilterGroups = id.to_filter_node(MC::COLUMN_ID).into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  // -- Execute the query
  match mm.dbx() {
    Dbx::Postgres(dbx_postgres) => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
      let res = dbx_postgres.fetch_optional(sqlx_query).await?;
      Ok(res)
    }
    #[allow(unreachable_patterns)]
    _ => Err(SqlError::InvalidDatabase("Need postgres database")),
  }
}

pub async fn pg_find_unique<MC, E, F>(mm: &ModelManager, filter: F) -> Result<Option<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
  F: Into<FilterGroups>,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(E::sea_column_refs());

  // condition from filter
  let filters: FilterGroups = filter.into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  // -- Execute the query
  match mm.dbx() {
    Dbx::Postgres(dbx_postgres) => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
      let entity = dbx_postgres.fetch_optional(sqlx_query).await?;

      Ok(entity)
    }
    #[allow(unreachable_patterns)]
    _ => Err(SqlError::InvalidDatabase("Need postgres database")),
  }
}

pub async fn pg_page<MC, E, F>(mm: &ModelManager, filter: F, page: Page) -> Result<PageResult<E>>
where
  MC: DbBmc,
  F: Into<FilterGroups>,
  E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
  E: HasSeaFields,
{
  let filter: FilterGroups = filter.into();
  let total = count::<MC, _>(mm, filter.clone()).await?;
  let result = pg_find_many::<MC, E, _>(mm, filter, Some(page)).await?;

  Ok(PageResult::new(total, result))
}
