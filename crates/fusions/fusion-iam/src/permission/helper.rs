use crate::pb::fusion_iam::v1::PermissionDto;

use super::Permission;

impl From<Permission> for PermissionDto {
  fn from(value: Permission) -> Self {
    Self {
      id: value.id,
      code: value.code,
      description: value.description,
      resource: value.resource,
      action: value.action,
      cid: value.cid,
      ctime: value.ctime.timestamp_millis(),
      mid: value.mid,
      mtime: value.mtime.map(|t| t.timestamp_millis()),
    }
  }
}
