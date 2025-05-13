use sea_query::{Condition, Query, SelectStatement, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use sqlx::{FromRow, sqlite::SqliteRow};

use crate::{
  ModelManager, Result, SqlError,
  base::{DbBmc, compute_list_options, count},
  field::HasSeaFields,
  filter::{FilterGroups, ListOptions},
  id::Id,
  page::PageResult,
  store::Dbx,
};

#[tracing::instrument(skip(mm, filter))]
pub async fn sqlite_find_first<MC, E, F>(mm: &ModelManager, filter: F) -> Result<Option<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
  E: HasSeaFields,
  F: Into<FilterGroups>,
{
  let list = sqlite_find_many::<MC, E, F>(mm, filter, None).await?;
  Ok(list.into_iter().next())
}

#[tracing::instrument(skip(mm, filter, list_options))]
pub async fn sqlite_find_many<MC, E, F>(
  mm: &ModelManager,
  filter: F,
  list_options: Option<ListOptions>,
) -> Result<Vec<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
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

  // list options
  let list_options = compute_list_options::<MC>(list_options)?;
  list_options.apply_to_sea_query(&mut query);

  // -- Execute the query
  match mm.dbx() {
    Dbx::Sqlite(dbx) => {
      let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
      let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
      let entities = dbx.fetch_all(sqlx_query).await?;
      Ok(entities)
    }
    #[allow(unreachable_patterns)]
    _ => Err(SqlError::InvalidDatabase("Need sqlite database")),
  }
}

#[tracing::instrument(skip(mm, f))]
pub async fn sqlite_find_many_on<MC, E, F>(mm: &ModelManager, f: F) -> Result<Vec<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
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
    Dbx::Sqlite(dbx) => {
      let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
      let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
      let entities = dbx.fetch_all(sqlx_query).await?;
      Ok(entities)
    }
    #[allow(unreachable_patterns)]
    _ => Err(SqlError::InvalidDatabase("Need sqlite database")),
  }
}

#[tracing::instrument(skip(mm, id))]
pub async fn sqlite_find_by_id<MC, E>(mm: &ModelManager, id: Id) -> Result<E>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
  E: HasSeaFields,
{
  // -- Build the query
  let mut query = Query::select();
  query.from(MC::table_ref()).columns(E::sea_column_refs());

  // condition from filter
  let filters: FilterGroups = id.to_filter_node("id").into();
  let cond: Condition = filters.try_into()?;
  query.cond_where(cond);

  // -- Execute the query
  match mm.dbx() {
    Dbx::Sqlite(dbx) => {
      let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
      let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
      let res = dbx.fetch_optional(sqlx_query).await?;
      match res {
        Some(entity) => Ok(entity),
        None => Err(SqlError::EntityNotFound { schema: MC::SCHEMA, entity: MC::TABLE, id }),
      }
    }
    #[allow(unreachable_patterns)]
    _ => Err(SqlError::InvalidDatabase("Need sqlite database")),
  }
}

#[tracing::instrument(skip(mm, filter))]
pub async fn sqlite_find_unique<MC, E, F>(mm: &ModelManager, filter: F) -> Result<Option<E>>
where
  MC: DbBmc,
  E: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
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
    Dbx::Sqlite(dbx) => {
      let (sql, values) = query.build_sqlx(SqliteQueryBuilder);
      let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
      let entity = dbx.fetch_optional(sqlx_query).await?;

      Ok(entity)
    }
    #[allow(unreachable_patterns)]
    _ => Err(SqlError::InvalidDatabase("Need sqlite database")),
  }
}

#[tracing::instrument(skip(mm, filter, list_options))]
pub async fn sqlite_page<MC, E, F>(mm: &ModelManager, filter: F, list_options: ListOptions) -> Result<PageResult<E>>
where
  MC: DbBmc,
  F: Into<FilterGroups>,
  E: for<'r> FromRow<'r, SqliteRow> + Unpin + Send,
  E: HasSeaFields,
{
  let filter: FilterGroups = filter.into();
  let total_size = count::<MC, _>(mm, filter.clone()).await?;
  let items = sqlite_find_many::<MC, E, _>(mm, filter, Some(list_options)).await?;

  Ok(PageResult::new(total_size, items))
}
