use fusionsql_core::page::StaticOrderBys;
use fusionsql_core::sea_utils::SIden;
use sea_query::{IntoIden, TableRef};

#[derive(Debug, Clone)]
pub struct BmcConfig {
  pub list_limit_default: u64,
  pub list_limit_max: u64,
  pub table: &'static str,
  pub schema: Option<&'static str>,
  pub column_id: &'static str,
  pub id_generated_by_db: bool,
  pub has_created_by: bool,
  pub has_created_at: bool,
  pub has_updated_by: bool,
  pub has_updated_at: bool,
  pub use_logical_deletion: bool,
  pub has_owner_id: bool,
  pub has_optimistic_lock: bool,
  pub order_bys: Option<StaticOrderBys>,
}

impl BmcConfig {
  pub fn new(table_name: &'static str, schema: Option<&'static str>) -> Self {
    Self {
      table: table_name,
      schema,
      list_limit_default: super::LIST_LIMIT_DEFAULT,
      list_limit_max: super::LIST_LIMIT_MAX,
      column_id: "id",
      id_generated_by_db: false,
      has_created_by: true,
      has_created_at: true,
      has_updated_by: true,
      has_updated_at: true,
      use_logical_deletion: false,
      has_owner_id: false,
      has_optimistic_lock: false,
      order_bys: None,
    }
  }

  pub fn new_table(table_name: &'static str) -> Self {
    Self::new(table_name, None)
  }

  pub fn with_list_limit_default(mut self, list_limit_default: u64) -> Self {
    self.list_limit_default = list_limit_default;
    self
  }

  pub fn with_list_limit_max(mut self, list_limit_max: u64) -> Self {
    self.list_limit_max = list_limit_max;
    self
  }

  pub fn with_column_id(mut self, column_id: &'static str) -> Self {
    self.column_id = column_id;
    self
  }

  pub fn with_id_generated_by_db(mut self, id_generated_by_db: bool) -> Self {
    self.id_generated_by_db = id_generated_by_db;
    self
  }

  pub fn with_has_created_by(mut self, has_created_by: bool) -> Self {
    self.has_created_by = has_created_by;
    self
  }

  pub fn with_has_created_at(mut self, has_created_at: bool) -> Self {
    self.has_created_at = has_created_at;
    self
  }

  pub fn with_has_updated_by(mut self, has_updated_by: bool) -> Self {
    self.has_updated_by = has_updated_by;
    self
  }

  pub fn with_has_updated_at(mut self, has_updated_at: bool) -> Self {
    self.has_updated_at = has_updated_at;
    self
  }

  pub fn with_use_logical_deletion(mut self, use_logical_deletion: bool) -> Self {
    self.use_logical_deletion = use_logical_deletion;
    self
  }

  pub fn with_has_owner_id(mut self, has_owner_id: bool) -> Self {
    self.has_owner_id = has_owner_id;
    self
  }

  pub fn with_has_optimistic_lock(mut self, has_optimistic_lock: bool) -> Self {
    self.has_optimistic_lock = has_optimistic_lock;
    self
  }

  pub fn with_order_bys(mut self, order_bys: Option<StaticOrderBys>) -> Self {
    self.order_bys = order_bys;
    self
  }

  pub fn table_ref(&self) -> TableRef {
    match self.schema {
      Some(schema) => TableRef::SchemaTable(SIden(schema).into_iden(), SIden(self.table).into_iden()),
      None => TableRef::Table(SIden(self.table).into_iden()),
    }
  }

  pub fn qualified_table(&self) -> (&'static str, &'static str) {
    (self.schema.unwrap_or("public"), self.table)
  }

  pub fn qualified_table_name(&self) -> String {
    match self.schema {
      Some(schema) => format!("{}.{}", schema, self.table),
      None => self.table.to_string(),
    }
  }
}

// /// 注意，暂未使用
// #[derive(Debug, Clone, Default)]
// pub struct DynBmcConfig {
//   pub order_bys: Option<OrderBys>,
// }

// impl DynBmcConfig {
//   pub fn with_order_bys(mut self, order_bys: Option<OrderBys>) -> Self {
//     self.order_bys = order_bys;
//     self
//   }
// }

/// The DbBmc trait must be implemented for the Bmc struct of an entity.
/// It specifies meta information such as the table name,
/// whether the table has timestamp columns (created_by, created_at, updated_by, updated_at), and more as the
/// code evolves.
///
/// Note: This trait should not be confused with the BaseCrudBmc trait, which provides
///       common default CRUD BMC functions for a given Bmc/Entity.
pub trait DbBmc {
  fn _bmc_config() -> &'static BmcConfig;
  // fn _dynamic_config() -> DynBmcConfig {
  //   DynBmcConfig::default()
  // }
}
