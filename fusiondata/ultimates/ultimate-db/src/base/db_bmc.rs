use modql::SIden;
use sea_query::{IntoIden, TableRef};

/// The DbBmc trait must be implemented for the Bmc struct of an entity.
/// It specifies meta information such as the table name,
/// whether the table has timestamp columns (cid, ctime, mid, mtime), and more as the
/// code evolves.
///
/// Note: This trait should not be confused with the BaseCrudBmc trait, which provides
///       common default CRUD BMC functions for a given Bmc/Entity.
pub trait DbBmc {
  const TABLE: &'static str;
  const SCHEMA: &'static str = "public";
  const LIST_LIMIT_DEFAULT: i64 = super::LIST_LIMIT_DEFAULT;
  const LIST_LIMIT_MAX: i64 = super::LIST_LIMIT_MAX;

  fn table_ref() -> TableRef {
    TableRef::SchemaTable(SIden(Self::SCHEMA).into_iden(), SIden(Self::TABLE).into_iden())
  }

  fn qualified_table() -> (&'static str, &'static str) {
    (Self::SCHEMA, Self::TABLE)
  }

  /// Specifies that the table for this Bmc has timestamps (cid, ctime, mid, mtime) columns.
  /// This will allow the code to update those as needed.
  ///
  /// default: true
  fn has_creation_timestamps() -> bool {
    true
  }

  /// default: true
  fn has_modification_timestamps() -> bool {
    true
  }

  /// 是否使用逻辑删除
  ///
  /// default: false
  fn use_logical_deletion() -> bool {
    false
  }

  /// Specifies if the entity table managed by this BMC
  /// has an `owner_id` column that needs to be set on create (by default ctx.user_id).
  ///
  /// default: false
  fn has_owner_id() -> bool {
    false
  }

  /// 乐观锁
  /// default: false
  fn has_optimistic_lock() -> bool {
    false
  }

  /// 是否过滤用 column id
  /// default: false
  fn filter_column_id() -> bool {
    false
  }
}
