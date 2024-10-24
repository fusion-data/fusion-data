use ultimate::{DataError, Result};
use ultimate_api::v1::PagePayload;
use ultimate_db::modql::filter::{OpValInt32, OpValString};

use crate::pb::fusion_iam::v1::{FilterRoleDto, PageRoleResponse, RoleDto, RoleStatus, UpdateRoleDto};

use super::{Role, RoleFilter, RoleForUpdate};

impl From<Role> for RoleDto {
  fn from(value: Role) -> Self {
    Self {
      id: value.id,
      name: value.name,
      description: value.description,
      status: value.status.into(),
      cid: value.cid,
      ctime: value.ctime.timestamp_millis(),
      mid: value.mid,
      mtime: value.mtime.map(|t| t.timestamp_millis()),
    }
  }
}

impl TryFrom<UpdateRoleDto> for RoleForUpdate {
  type Error = DataError;

  fn try_from(value: UpdateRoleDto) -> Result<Self> {
    let status = match value.status {
      Some(status) => Some(
        RoleStatus::try_from(status)
          .map_err(|e| DataError::bad_request(format!("Unknown 'status' enum value: {}", e.0)))?,
      ),
      _ => None,
    };
    Ok(Self { name: value.name, description: value.description, status })
  }
}

impl From<FilterRoleDto> for RoleFilter {
  fn from(value: FilterRoleDto) -> Self {
    let status = if value.status.is_empty() { None } else { Some(OpValInt32::In(value.status).into()) };
    Self {
      name: value.name.map(|name| OpValString::Contains(name).into()),
      description: value.description.map(|desc| OpValString::StartsWith(desc).into()),
      status,
    }
  }
}

impl From<PagePayload<Role>> for PageRoleResponse {
  fn from(value: PagePayload<Role>) -> Self {
    Self { page: Some(value.page), items: value.items.into_iter().map(Into::into).collect() }
  }
}
