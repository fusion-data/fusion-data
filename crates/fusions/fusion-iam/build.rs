use std::{env, path::PathBuf};

static BASE_PACKAGE: &str = ".fusion_iam.v1";

static ENUM_ATTR: &str =
  "#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, enum_iterator::Sequence, sqlx::Type)]";
static MESSAGE_ATTR: &str = "#[derive(serde::Serialize, serde::Deserialize)]";
static MODQL_MESSAGE_ATTR: &str = "#[derive(modql::field::Fields)]";

fn main() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  let enums = ["UserStatus", "Gender", "RoleStatus"];
  // let messages = [
  //   "SigninRequest",
  //   "SigninReplay",
  //   "UserDto",
  //   "FindUserRequest",
  //   "CreateUserRequest",
  //   "UpdateUserRequest",
  //   "PageUserRequest",
  //   "FilterUserRequest",
  //   "PageUserResponse",
  //   "DeleteUserResponse",
  //   "UserResponse",
  //   "Role",
  //   "AssignRoleRequest",
  //   "AssignPermissionRequest",
  //   "CreateRoleRequest",
  //   "UpdateRoleRequest",
  //   "DeleteRoleRequest",
  //   "GetRoleRequest",
  //   "RoleResponse",
  //   "DeleteRoleResponse",
  //   "PermissionDto",
  //   "CreatePermissionRequest",
  //   "UpdatePermissionRequest",
  //   "DeletePermissionRequest",
  //   "PermissionResponse",
  //   "DeletePermissionResponse",
  // ];
  let field_messages = ["CreateRoleDto"];

  let mut iam_b = tonic_build::configure()
    .emit_rerun_if_changed(true)
    .file_descriptor_set_path(out_dir.join("fusion_iam_descriptor.bin"))
    // .compile_well_known_types(true)
    .extern_path(".ultimate_api", "::ultimate_api");

  iam_b = enums.iter().fold(iam_b, |b, e| b.enum_attribute(format!("{}.{}", BASE_PACKAGE, e), ENUM_ATTR));
  // builder = messages.iter().fold(builder, |b, m| b.message_attribute(format!("{}.{}", BASE_PACKAGE, m), MESSAGE_ATTR));
  iam_b = field_messages
    .iter()
    .fold(iam_b, |b, m| b.message_attribute(format!("{}.{}", BASE_PACKAGE, m), MODQL_MESSAGE_ATTR));

  iam_b
    .compile(
      &[
        "proto/fusion_iam/v1/auth.proto",
        "proto/fusion_iam/v1/user.proto",
        "proto/fusion_iam/v1/role.proto",
        "proto/fusion_iam/v1/permission.proto",
        "proto/fusion_iam/v1/access_control.proto",
      ],
      &["proto"],
    )
    .unwrap();
}
