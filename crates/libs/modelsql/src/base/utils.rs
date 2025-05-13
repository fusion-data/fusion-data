use crate::{
  SqlError,
  field::{SeaField, SeaFields},
  filter::ListOptions,
  store::dbx::DbxType,
};
use sea_query::{DeleteStatement, DynIden, InsertStatement, IntoIden, SelectStatement, UpdateStatement, WithQuery};
use sea_query_binder::{SqlxBinder, SqlxValues};
use ultimate_common::ctx::Ctx;

use crate::{
  Result,
  base::{CommonIden, DbBmc, TimestampIden},
};

pub fn build_sqlx_for_update(dbx_type: &DbxType, query: UpdateStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxType::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxType::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_select(dbx_type: &DbxType, query: SelectStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxType::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxType::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_query(dbx_type: &DbxType, query: WithQuery) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxType::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxType::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_delete(dbx_type: &DbxType, query: DeleteStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxType::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxType::Sqlite => {
      let (sql, values) = query.build_sqlx(sea_query::SqliteQueryBuilder);
      (sql, values)
    }
  }
}

pub fn build_sqlx_for_insert(dbx_type: &DbxType, query: InsertStatement) -> (String, SqlxValues) {
  match dbx_type {
    #[cfg(feature = "with-postgres")]
    DbxType::Postgres => {
      let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
      (sql, values)
    }
    #[cfg(feature = "with-sqlite")]
    DbxType::Sqlite => {
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
  if MC::has_owner_id() {
    fields.push(SeaField::new(CommonIden::OwnerId.into_iden(), ctx.uid()));
  }

  if MC::has_creation_timestamps() {
    fields = add_timestamps_for_create(fields, ctx);
  }

  if MC::filter_column_id() {
    fields = SeaFields::new(fields.into_iter().filter(|f| f.iden.to_string() != "id").collect());
  }

  fields
}

/// This method must be calledwhen a Model Controller plans to update its entity.
pub fn prep_fields_for_update<MC>(fields: SeaFields, ctx: &Ctx) -> SeaFields
where
  MC: DbBmc,
{
  if MC::has_creation_timestamps() { add_timestamps_for_update(fields, ctx) } else { fields }
}

pub fn clear_id_from_fields<MC>(fields: SeaFields) -> SeaFields {
  let mut fields = fields.into_vec();
  fields.retain(|f| f.iden != CommonIden::Id.into_iden());
  SeaFields::new(fields)
}

fn _exists_in_fields(fields: &[SeaField], iden: DynIden) -> bool {
  // let iden = iden.into_iden();
  fields.iter().any(|f| f.iden == iden)
}

/// Update the timestamps info for create
/// (e.g., cid, ctime, and mid, mtime will be updated with the same values)
fn add_timestamps_for_create(fields: SeaFields, ctx: &Ctx) -> SeaFields {
  let mut fields = fields.into_vec();
  if !_exists_in_fields(&fields, TimestampIden::Cid.into_iden()) {
    fields.push(SeaField::new(TimestampIden::Cid, ctx.uid()));
  }
  if !_exists_in_fields(&fields, TimestampIden::Ctime.into_iden()) {
    fields.push(SeaField::new(TimestampIden::Ctime, *ctx.req_time()));
  }
  SeaFields::new(fields)
}

/// Update the timestamps info only for update.
/// (.e.g., only mid, mtime will be udpated)
fn add_timestamps_for_update(fields: SeaFields, ctx: &Ctx) -> SeaFields {
  let mut fields = fields.into_vec();
  if !_exists_in_fields(&fields, TimestampIden::Mid.into_iden()) {
    fields.push(SeaField::new(TimestampIden::Mid, ctx.uid()));
  }
  if !_exists_in_fields(&fields, TimestampIden::Mtime.into_iden()) {
    fields.push(SeaField::new(TimestampIden::Mtime, *ctx.req_time()));
  }
  SeaFields::new(fields)
}

// #[tracing::instrument(skip(list_options))]
pub fn compute_list_options<MC>(list_options: Option<ListOptions>) -> Result<ListOptions>
where
  MC: DbBmc,
{
  if let Some(mut list_options) = list_options {
    // Validate the limit.
    if let Some(limit) = list_options.limit {
      if limit > MC::LIST_LIMIT_MAX {
        return Err(SqlError::ListLimitOverMax { max: MC::LIST_LIMIT_MAX, actual: limit });
      } else if limit < 1 {
        return Err(SqlError::ListLimitUnderMin { min: 1, actual: limit });
      }
    } else {
      // Set the default limit if no limit
      list_options.limit = Some(MC::LIST_LIMIT_DEFAULT);
    }
    if let Some(page) = list_options.page {
      if page < 1 {
        return Err(SqlError::ListPageUnderMin { min: 1, actual: page });
      }
    }
    Ok(list_options)
  } else {
    // When None, return default
    Ok(ListOptions { limit: Some(MC::LIST_LIMIT_DEFAULT), ..Default::default() })
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
