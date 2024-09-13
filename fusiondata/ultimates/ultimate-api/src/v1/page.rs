use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, PartialEq, ::prost::Message, Serialize)]
pub struct OperationResponse {
  #[prost(int32, tag = "1")]
  pub code: i32,
  #[prost(string, optional, tag = "2")]
  pub message: ::core::option::Option<::prost::alloc::string::String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PagePayload<T> {
  pub page: Page,
  pub items: Vec<T>,
}

impl<T> PagePayload<T> {
  pub fn new(page: Page, items: Vec<T>) -> Self {
    Self { page, items }
  }
}

#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct Pagination {
  #[prost(int64, tag = "1")]
  pub page: i64,
  #[prost(int64, tag = "2")]
  pub page_size: i64,
  #[prost(message, repeated, tag = "3")]
  pub sort_bys: ::prost::alloc::vec::Vec<SortBy>,
  #[prost(int64, optional, tag = "4")]
  pub offset: ::core::option::Option<i64>,
}

impl Pagination {
  pub fn page(&self) -> i64 {
    self.page
  }

  pub fn page_size(&self) -> i64 {
    self.page_size
  }

  pub fn sort_bys(&self) -> Vec<&SortBy> {
    self.sort_bys.iter().collect()
  }

  pub fn offset_value(&self) -> i64 {
    if let Some(offset) = self.offset {
      return offset;
    }
    let page = self.page();
    let page_size = self.page_size();
    if page < 2 {
      return 0;
    }
    page_size * (page - 1)
  }

  pub fn new_default() -> Self {
    Self {
      page: default_page(),
      page_size: default_page_size(),
      sort_bys: Default::default(),
      offset: Default::default(),
    }
  }
}

#[derive(Clone, Copy, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct Page {
  #[prost(int64, tag = "1")]
  pub page: i64,
  #[prost(int64, tag = "2")]
  pub page_size: i64,
  #[prost(int64, tag = "3")]
  pub total_size: i64,
  #[prost(int64, tag = "4")]
  pub total_page: i64,
}

impl Page {
  pub fn new(pagination: &Pagination, total_size: i64) -> Self {
    let page = pagination.page;
    let page_size = pagination.page_size;
    let total_page = if total_size == 0 { 0 } else { (total_size + page_size - 1) / page_size };
    Self { page, page_size, total_size, total_page }
  }
}

#[derive(Clone, PartialEq, ::prost::Message, Serialize, Deserialize)]
pub struct SortBy {
  #[prost(string, tag = "1")]
  pub f: ::prost::alloc::string::String,
  #[prost(enumeration = "SortDirection", tag = "2")]
  pub d: i32,
}

#[cfg(feature = "modql")]
impl From<SortBy> for modql::filter::OrderBy {
  fn from(value: SortBy) -> Self {
    match value.d.try_into().unwrap_or_default() {
      SortDirection::ASC | SortDirection::UNSPECIFIED => modql::filter::OrderBy::Asc(value.f),
      SortDirection::DESC => modql::filter::OrderBy::Desc(value.f),
    }
  }
}

#[cfg(feature = "modql")]
impl From<&SortBy> for modql::filter::OrderBy {
  fn from(value: &SortBy) -> Self {
    match value.d.try_into().unwrap_or_default() {
      SortDirection::ASC | SortDirection::UNSPECIFIED => modql::filter::OrderBy::Asc(value.f.clone()),
      SortDirection::DESC => modql::filter::OrderBy::Desc(value.f.clone()),
    }
  }
}

#[derive(
  Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration, Serialize_repr, Deserialize_repr,
)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum SortDirection {
  UNSPECIFIED = 0,
  ASC = 1,
  DESC = 2,
}

impl SortDirection {
  /// String value of the enum field names used in the ProtoBuf definition.
  ///
  /// The values are not transformed in any way and thus are considered stable
  /// (if the ProtoBuf definition does not change) and safe for programmatic use.
  pub fn as_str_name(&self) -> &'static str {
    match self {
      SortDirection::ASC => "ASC",
      SortDirection::DESC => "DESC",
      SortDirection::UNSPECIFIED => "UNSPECIFIED",
    }
  }
  /// Creates an enum from field names used in the ProtoBuf definition.
  pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
    match value {
      "ASC" => Some(Self::ASC),
      "DESC" => Some(Self::DESC),
      _ => None,
    }
  }
}

#[cfg(feature = "modql")]
impl From<Pagination> for modql::filter::ListOptions {
  fn from(value: Pagination) -> Self {
    let offset = Some(value.offset_value());
    let limit = Some(if value.page_size > 0 { value.page_size } else { default_page_size() });
    let order_bys = Some(modql::filter::OrderBys::new(value.sort_bys.into_iter().map(Into::into).collect()));
    modql::filter::ListOptions { limit, offset, order_bys }
  }
}

#[cfg(feature = "modql")]
impl From<&Pagination> for modql::filter::ListOptions {
  fn from(value: &Pagination) -> Self {
    let offset = Some(value.offset_value());
    let limit = Some(if value.page_size > 0 { value.page_size } else { default_page_size() });
    let order_bys = Some(modql::filter::OrderBys::new(value.sort_bys.iter().map(|v| v.into()).collect()));
    modql::filter::ListOptions { limit, offset, order_bys }
  }
}

fn default_page() -> i64 {
  1
}

fn default_page_size() -> i64 {
  20
}
