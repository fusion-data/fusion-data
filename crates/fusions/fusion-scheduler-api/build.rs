use std::{env, path::PathBuf};

static BASE_PACKAGE: &str = ".fusion_scheduler_api.v1";

static MESSAGE_ATTR: &str = "#[derive(serde::Serialize, serde::Deserialize)]";
// static MODQL_MESSAGE_ATTR: &str = "#[derive(modql::field::Fields)]";
static ENUM_ATTR: &str = "#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]";
static ENUM_ITERATOR_ATTR: &str = r#"#[cfg_attr(feature = "enum-iterator", derive(enum_iterator::Sequence))]"#;
static ENUM_SQLX_ATTR: &str = r#"#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]"#;
fn main() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  let enum_list = [
    "InstanceTask.TaskKind",
    "TriggerDefinition.TriggerKind",
    "ProcessInstance.InstanceStatus",
    "ProcessDefinition.ProcessStatus",
    "SchedNode.NodeStatus",
    "SchedNode.NodeKind",
  ];
  let oneof_list =
    ["TriggerDefinition.schedule", "CreateTriggerDefinitionRequest.schedule", "UpdateTriggerRequest.schedule"];

  let mut iam_b = tonic_build::configure()
    .emit_rerun_if_changed(true)
    .extern_path(".ultimate_api.v1", "::ultimate_api::v1")
    .message_attribute(BASE_PACKAGE, MESSAGE_ATTR)
    // .bytes(["."])
    .file_descriptor_set_path(out_dir.join("fusion_scheduler_api_descriptor.bin"));

  iam_b = enum_list.iter().fold(iam_b, |b, e| {
    b.enum_attribute(format!("{}.{}", BASE_PACKAGE, e), ENUM_ITERATOR_ATTR)
      .enum_attribute(format!("{}.{}", BASE_PACKAGE, e), ENUM_SQLX_ATTR)
      .enum_attribute(format!("{}.{}", BASE_PACKAGE, e), ENUM_ATTR)
  });
  iam_b = oneof_list.iter().fold(iam_b, |b, e| {
    b.type_attribute(format!("{}.{}", BASE_PACKAGE, e), "#[derive(serde::Serialize, serde::Deserialize)]")
  });
  // builder = messages.iter().fold(builder, |b, m| b.message_attribute(format!("{}.{}", BASE_PACKAGE, m), MESSAGE_ATTR));
  // iam_b = field_messages
  //   .iter()
  //   .fold(iam_b, |b, m| b.message_attribute(format!("{}.{}", BASE_PACKAGE, m), MODQL_MESSAGE_ATTR));

  iam_b
    .compile_protos(
      &["proto/fusion_scheduler_api/v1/scheduler_api.proto"],
      &["proto", "../../ultimates/ultimate-api/proto"],
    )
    .unwrap();
}
