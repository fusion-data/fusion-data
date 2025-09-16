//! Protocol buffer definitions for Fusion IAM service.
//!
//! This module contains Rust struct definitions that correspond to the
//! original Protocol Buffer definitions in the hetuiam project.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

/// Empty response message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Empty {}

/// User module containing user-related structures
pub mod user {
  use super::*;

  /// User status enumeration
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
  #[repr(i32)]
  pub enum UserStatus {
    Unspecified = 0,
    Disabled = 99,
    Enabled = 100,
  }

  impl Default for UserStatus {
    fn default() -> Self {
      UserStatus::Unspecified
    }
  }

  impl From<i32> for UserStatus {
    fn from(value: i32) -> Self {
      match value {
        99 => UserStatus::Disabled,
        100 => UserStatus::Enabled,
        _ => UserStatus::Unspecified,
      }
    }
  }

  /// Gender enumeration
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
  #[repr(i32)]
  pub enum Gender {
    Unspecified = 0,
    Male = 1,
    Female = 2,
  }

  impl Default for Gender {
    fn default() -> Self {
      Gender::Unspecified
    }
  }

  impl From<i32> for Gender {
    fn from(value: i32) -> Self {
      match value {
        1 => Gender::Male,
        2 => Gender::Female,
        _ => Gender::Unspecified,
      }
    }
  }

  /// User data transfer object
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct UserDto {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: UserStatus,
    pub gender: Gender,
    pub cid: i64,
    pub ctime: DateTime<FixedOffset>,
    pub mid: Option<i64>,
    pub mtime: Option<DateTime<FixedOffset>>,
  }

  /// Request to assign user to roles
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct AssignUserToRolesRequest {
    pub user_id: i64,
    pub role_ids: Vec<i64>,
  }

  /// Find user request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct FindUserRequest {
    pub id: i64,
  }

  /// Create user request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct CreateUserRequest {
    pub returning_payload: bool,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<i32>,
    pub password: Option<String>,
  }

  /// Create user response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct CreateUserResponse {
    pub data: CreateUserResponseData,
  }

  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  #[serde(untagged)]
  pub enum CreateUserResponseData {
    Id(i64),
    User(UserDto),
  }

  /// Update user request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct UpdateUserRequest {
    pub returning_payload: bool,
    pub id: i64,
    pub name: Option<String>,
    pub status: Option<i32>,
  }

  /// User response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct UserResponse {
    pub user: UserDto,
  }

  /// Delete user response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct DeleteUserResponse {}

  /// Page user request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct PageUserRequest {
    pub pagination: ultimate_api::v1::page::Pagination,
    pub filter: Vec<FilterUserRequest>,
  }

  /// Filter user request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct FilterUserRequest {
    pub name: Vec<ultimate_api::v1::ql::ValString>,
    pub email: Vec<ultimate_api::v1::ql::ValString>,
    pub phone: Vec<ultimate_api::v1::ql::ValString>,
    pub status: Vec<ultimate_api::v1::ql::ValInt32>,
    pub gender: Vec<ultimate_api::v1::ql::ValInt32>,
    pub cid: Vec<ultimate_api::v1::ql::ValInt64>,
    pub ctime: Vec<ultimate_api::v1::ql::ValString>,
    pub id: Vec<ultimate_api::v1::ql::ValInt64>,
    pub mid: Vec<ultimate_api::v1::ql::ValInt64>,
    pub mtime: Vec<ultimate_api::v1::ql::ValString>,
  }

  /// Page user response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct PageUserResponse {
    pub page: ultimate_api::v1::page::Page,
    pub items: Vec<UserDto>,
  }
}

/// Role module containing role-related structures
pub mod role {
  use modelsql::Fields;

  use super::*;

  /// Role status enumeration
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
  #[repr(i32)]
  pub enum RoleStatus {
    Unspecified = 0,
    Disabled = 99,
    Enabled = 100,
  }

  impl From<i32> for RoleStatus {
    fn from(value: i32) -> Self {
      match value {
        99 => RoleStatus::Disabled,
        100 => RoleStatus::Enabled,
        _ => RoleStatus::Unspecified,
      }
    }
  }

  /// Role data transfer object
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct RoleDto {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub status: RoleStatus,
    pub cid: i64,
    pub ctime: DateTime<FixedOffset>,
    pub mid: Option<i64>,
    pub mtime: Option<DateTime<FixedOffset>>,
  }

  /// Request to assign role to permissions
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct AssignRoleToPermissionsRequest {
    pub role_id: i64,
    pub permission_ids: Vec<i64>,
  }

  /// Get role request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct GetRoleRequest {
    pub field_mask: Option<google::protobuf::field_mask::FieldMask>,
    pub id: i64,
  }

  /// Create role request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct CreateRoleRequest {
    pub field_mask: Option<google::protobuf::field_mask::FieldMask>,
    pub create_role: CreateRoleDto,
    pub permission_ids: Vec<i64>,
  }

  /// Create role DTO
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Fields)]
  pub struct CreateRoleDto {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<RoleStatus>,
  }

  /// Update role request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct UpdateRoleRequest {
    pub field_mask: Option<google::protobuf::field_mask::FieldMask>,
    pub id: i64,
    pub dto: UpdateRoleDto,
  }

  /// Update role DTO
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct UpdateRoleDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<RoleStatus>,
  }

  /// Delete role request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct DeleteRoleRequest {
    pub id: i64,
  }

  /// Role response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct RoleResponse {
    pub role: RoleDto,
    pub permissions: Vec<super::permission::PermissionDto>,
  }

  /// Delete role response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct DeleteRoleResponse {}

  /// Filter role DTO
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct FilterRoleDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Vec<RoleStatus>,
  }

  /// Page role request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct PageRoleRequest {
    pub pagination: ultimate_api::v1::page::Pagination,
    pub filter: Vec<FilterRoleDto>,
    pub role_perm_filter: super::permission::RolePermissionFilterDto,
  }

  /// Page role response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct PageRoleResponse {
    pub page: ultimate_api::v1::page::Page,
    pub items: Vec<RoleDto>,
  }
}

/// Permission module containing permission-related structures
pub mod permission {
  use super::*;

  /// Permission data transfer object
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct PermissionDto {
    pub id: i64,
    pub code: String,
    pub description: String,
    pub resource: String,
    pub action: String,
    pub cid: i64,
    pub ctime: DateTime<FixedOffset>,
    pub mid: Option<i64>,
    pub mtime: Option<DateTime<FixedOffset>>,
  }

  /// Request to assign permission to roles
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct AssignPermmissionToRolesRequest {
    pub permission_id: i64,
    pub role_ids: Vec<i64>,
  }

  /// Create permission request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct CreatePermissionRequest {
    pub field_mask: Option<google::protobuf::field_mask::FieldMask>,
    pub dto: CreatePermissionDto,
  }

  /// Create permission DTO
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct CreatePermissionDto {
    pub code: String,
    pub description: Option<String>,
    pub resource: String,
    pub action: String,
  }

  /// Update permission DTO
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct UpdatePermissionDto {
    pub code: Option<String>,
    pub description: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
  }

  /// Get permission request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct GetPermissionRequest {
    pub id: i64,
  }

  /// Update permission request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct UpdatePermissionRequest {
    pub field_mask: Option<google::protobuf::field_mask::FieldMask>,
    pub id: i64,
    pub dto: UpdatePermissionDto,
  }

  /// Delete permission request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct DeletePermissionRequest {
    pub id: i64,
  }

  /// Permission response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct PermissionResponse {
    pub id: i64,
    pub permission: PermissionDto,
  }

  /// Delete permission response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct DeletePermissionResponse {}

  /// Filter permission DTO
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct FilterPermissionDto {
    pub code: Option<String>,
    pub description: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
  }

  /// Role permission filter DTO
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct RolePermissionFilterDto {
    pub role_id: Option<i64>,
    pub permission_id: Option<i64>,
  }

  /// Page permission request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct PagePermissionRequest {
    pub pagination: ultimate_api::v1::page::Pagination,
    pub filter: Vec<FilterPermissionDto>,
    pub role_perm_filter: RolePermissionFilterDto,
  }

  /// Page permission response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct PagePermissionResponse {
    pub page: ultimate_api::v1::page::Page,
    pub items: Vec<PermissionDto>,
  }
}

/// Access control module containing policy-related structures
pub mod access_control {
  use super::*;

  /// Policy statement data transfer object
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct PolicyStatementDto {
    pub id: String,
    pub description: Option<String>,
    pub policy: String,
    pub status: i32,
    pub cid: i64,
    pub ctime: DateTime<FixedOffset>,
    pub mid: Option<i64>,
    pub mtime: Option<DateTime<FixedOffset>>,
  }

  /// Create policy request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct CreatePolicyRequest {
    pub policy: String,
    pub description: Option<String>,
  }

  /// Create policy response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct CreatePolicyResponse {
    pub id: String,
    pub policy_statement: PolicyStatementDto,
  }

  /// Get policy request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct GetPolicyRequest {
    pub id: String,
  }

  /// Get policy response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct GetPolicyResponse {
    pub policy_statement: PolicyStatementDto,
  }

  /// Update policy request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct UpdatePolicyRequest {
    pub id: String,
    pub policy: Option<String>,
    pub status: Option<i32>,
  }

  /// Update policy response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct UpdatePolicyResponse {
    pub policy_statement: PolicyStatementDto,
  }

  /// Delete policy request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct DeletePolicyRequest {
    pub id: String,
  }

  /// Delete policy response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct DeletePolicyResponse {}
}

/// Auth module containing authentication-related structures
pub mod auth {
  use super::*;

  /// Token kind enumeration
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  #[repr(i32)]
  pub enum TokenKind {
    Unspecified = 0,
    Bearer = 1,
  }

  impl Default for TokenKind {
    fn default() -> Self {
      TokenKind::Unspecified
    }
  }

  impl From<i32> for TokenKind {
    fn from(value: i32) -> Self {
      match value {
        1 => TokenKind::Bearer,
        _ => TokenKind::Unspecified,
      }
    }
  }

  /// Signin request
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct SigninRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: String,
  }

  /// Signin response
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct SigninResponse {
    pub token: String,
    pub token_kind: TokenKind,
  }
}

/// Google protobuf types
pub mod google {
  pub mod protobuf {
    pub mod field_mask {
      use serde::{Deserialize, Serialize};

      /// Field mask for partial updates
      #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
      pub struct FieldMask {
        pub paths: Vec<String>,
      }
    }
  }
}

/// Ultimate API types
pub mod ultimate_api {
  pub mod v1 {
    pub mod page {
      use serde::{Deserialize, Serialize};

      /// Pagination information
      #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
      pub struct Pagination {
        pub page: Option<i32>,
        pub size: Option<i32>,
        pub offset: Option<i32>,
      }

      /// Page information
      #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
      pub struct Page {
        pub total: i64,
        pub page: i32,
        pub size: i32,
        pub pages: i32,
      }
    }

    pub mod ql {
      use serde::{Deserialize, Serialize};

      /// String value with operator
      #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
      pub struct ValString {
        pub op: Option<String>,
        pub value: Option<String>,
      }

      /// 32-bit integer value with operator
      #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
      pub struct ValInt32 {
        pub op: Option<String>,
        pub value: Option<i32>,
      }

      /// 64-bit integer value with operator
      #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
      pub struct ValInt64 {
        pub op: Option<String>,
        pub value: Option<i64>,
      }
    }
  }
}

pub use access_control::*;
pub use auth::*;
pub use permission::*;
pub use role::*;
/// Re-export commonly used types
pub use user::*;
