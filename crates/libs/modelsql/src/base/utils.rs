use chrono::{DateTime, Utc};
use fusion_corelib::ctx::Ctx;
use modelsql_core::filter::Page;
use sea_query::{DeleteStatement, InsertStatement, IntoIden, SelectStatement, UpdateStatement, WithQuery};
#[cfg(any(feature = "with-postgres", feature = "with-sqlite"))]
use sea_query_binder::{SqlxBinder, SqlxValues};

use crate::{
  Result, SqlError,
  base::{CommonIden, DbBmc, TimestampIden},
  field::{SeaField, SeaFields},
  store::dbx::DbxProvider,
};

pub fn build_sqlx_for_update(dbx_type: &DbxProvider, query: UpdateStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_select(dbx_type: &DbxProvider, query: SelectStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_query(dbx_type: &DbxProvider, query: WithQuery) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_delete(dbx_type: &DbxProvider, query: DeleteStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_insert(dbx_type: &DbxProvider, query: InsertStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxProvider::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxProvider::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

/// This method must be called when a model controller intends to create its entity.
pub fn prep_fields_for_create<MC>(mut fields: SeaFields, ctx: &Ctx) -> SeaFields
where
  MC: DbBmc,
{
  fill_creations::<MC>(&mut fields, ctx);

  if MC::ID_GENERATED_BY_DB {
    fields = SeaFields::new(fields.into_iter().filter(|f| f.iden.to_string() != MC::COLUMN_ID).collect());
  }

  fields
}

/// This method must be called when a Model Controller plans to update its entity.
pub fn prep_fields_for_update<MC>(mut fields: SeaFields, ctx: &Ctx) -> SeaFields
where
  MC: DbBmc,
{
  fill_modifications::<MC>(&mut fields, ctx);
  fields
}

pub fn clear_id_from_fields<MC>(fields: SeaFields) -> SeaFields
where
  MC: DbBmc,
{
  let mut fields = fields.into_vec();
  fields.retain(|f| f.iden != MC::COLUMN_ID.into_iden());
  SeaFields::new(fields)
}

/// Update the creations info for create
/// (e.g., created_by, created_at, and updated_by, updated_at will be updated with the same values)
fn fill_creations<MC>(fields: &mut SeaFields, ctx: &Ctx)
where
  MC: DbBmc,
{
  if MC::_has_owner_id() {
    fields.push(SeaField::new(CommonIden::OwnerId.into_iden(), ctx.uid()));
  }
  if MC::_has_created_by() && !fields.exists(TimestampIden::CreatedBy.into_iden()) {
    fields.push(SeaField::new(TimestampIden::CreatedBy, ctx.uid()));
  }
  if MC::_has_created_at() && !fields.exists(TimestampIden::CreatedAt.into_iden()) {
    fields.push(SeaField::new(TimestampIden::CreatedAt, DateTime::<Utc>::from(*ctx.req_time())));
  }
}

/// Update the modifications info only for update.
/// (.e.g., only updated_by, updated_at will be updated)
fn fill_modifications<MC>(fields: &mut SeaFields, ctx: &Ctx)
where
  MC: DbBmc,
{
  if MC::_has_updated_by() && !fields.exists(TimestampIden::UpdatedBy.into_iden()) {
    fields.push(SeaField::new(TimestampIden::UpdatedBy, ctx.uid()));
  }
  if MC::_has_updated_at() && !fields.exists(TimestampIden::UpdatedAt.into_iden()) {
    fields.push(SeaField::new(TimestampIden::UpdatedAt, DateTime::<Utc>::from(*ctx.req_time())));
  }
}

pub fn compute_page<MC>(page: Option<Page>) -> Result<Page>
where
  MC: DbBmc,
{
  if let Some(mut page) = page {
    // Validate the limit.
    if let Some(limit) = page.limit {
      if limit > MC::LIST_LIMIT_MAX {
        return Err(SqlError::ListLimitOverMax { max: MC::LIST_LIMIT_MAX, actual: limit });
      } else if limit < 1 {
        return Err(SqlError::ListLimitUnderMin { min: 1, actual: limit });
      }
    } else {
      // Set the default limit if no limit
      page.limit = Some(MC::LIST_LIMIT_DEFAULT);
    }
    if let Some(page) = page.page
      && page < 1
    {
      return Err(SqlError::ListPageUnderMin { min: 1, actual: page });
    }
    Ok(page)
  } else {
    // When None, return default
    Ok(Page { limit: Some(MC::LIST_LIMIT_DEFAULT), ..Default::default() })
  }
}

/// 检查 sql execute 语句后受影响的数量
pub fn check_number_of_affected<MC>(expect_n: usize, return_n: u64) -> Result<u64>
where
  MC: DbBmc,
{
  // -- Check result
  if return_n as usize != expect_n {
    Err(SqlError::EntityNotFound {
      schema: MC::SCHEMA,
      entity: MC::TABLE,
      id: 0.into(), // Using 0 because multiple IDs could be not found, you may want to improve error handling here
    })
  } else {
    Ok(return_n)
  }
}
