use crate::pb::PermissionDto;

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
      ctime: value.ctime,
      mid: value.mid,
      mtime: value.mtime,
    }
  }
}
