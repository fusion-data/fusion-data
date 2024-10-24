use o2o::o2o;
use sea_query::enum_def;
use sqlx::prelude::FromRow;
use ultimate_api::v1::PagePayload;
use ultimate_common::time::UtcDateTime;
use ultimate_db::modql::{
  field::Fields,
  filter::{FilterNodes, OpValString, OpValsInt64, OpValsString},
};
use ultimate_db::DbRowType;

use crate::{
  pb::fusion_iam::v1::{CreatePermissionDto, FilterPermissionDto, PagePermissionResponse, UpdatePermissionDto},
  role::role_permission::RolePermissionFilter,
};

#[derive(Debug, FromRow, Fields)]
#[enum_def]
pub struct Permission {
  pub id: i64,
  pub code: String,
  pub description: String,
  pub resource: String,
  pub action: String,

  pub cid: i64,
  pub ctime: UtcDateTime,
  pub mid: Option<i64>,
  pub mtime: Option<UtcDateTime>,
}
impl DbRowType for Permission {}

#[derive(Debug, Fields, o2o)]
#[from_owned(CreatePermissionDto)]
pub struct PermissionForCreate {
  pub code: String,
  pub description: Option<String>,
  pub resource: String,
  pub action: String,
}

#[derive(Debug, Fields, o2o)]
#[from_owned(UpdatePermissionDto)]
pub struct PermissionForUpdate {
  pub code: Option<String>,
  pub description: Option<String>,
  pub resource: Option<String>,
  pub action: Option<String>,
}

#[derive(Debug, Clone, Default, FilterNodes)]
pub struct PermissionFilter {
  pub id: Option<OpValsInt64>,
  pub code: Option<OpValsString>,
  pub description: Option<OpValsString>,
  pub resource: Option<OpValsString>,
  pub action: Option<OpValsString>,
}

#[derive(Debug, Clone, Default)]
pub struct PermissionFilters {
  pub filter: Vec<PermissionFilter>,
  pub role_perm_filter: RolePermissionFilter,
}

impl From<FilterPermissionDto> for PermissionFilter {
  fn from(value: FilterPermissionDto) -> Self {
    Self {
      description: value.description.map(|v| OpValString::Eq(v).into()),
      resource: value.resource.map(|v| OpValString::Eq(v).into()),
      action: value.action.map(|v| OpValString::Eq(v).into()),
      code: value.code.map(|v| OpValString::Eq(v).into()),
      ..Default::default()
    }
  }
}

impl From<PagePayload<Permission>> for PagePermissionResponse {
  fn from(value: PagePayload<Permission>) -> Self {
    let items = value.items.into_iter().map(|v| v.into()).collect();
    Self { page: Some(value.page), items }
  }
}
