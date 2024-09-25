use std::{env, path::PathBuf};

static BASE_PACKAGE: &str = ".fusion_scheduler_api.v1";

static MESSAGE_ATTR: &str = "#[derive(serde::Serialize, serde::Deserialize)]";
// static MODQL_MESSAGE_ATTR: &str = "#[derive(modql::field::Fields)]";

fn main() {
  let _out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  let enum_reprs = ["JobDefinition.JobType"];
  let enums = ["TriggerDefinition.schedule", "UpdateTriggerRequest.schedule"];

  let mut iam_b = tonic_build::configure()
    .emit_rerun_if_changed(true)
    // .file_descriptor_set_path(out_dir.join("fusion_scheduler_api_descriptor.bin"))
    // .extern_path(".fusion_scheduler_api.v1", "::fusion_scheduler_api::v1");
    .message_attribute(BASE_PACKAGE, MESSAGE_ATTR)
    .bytes(["."]);

  iam_b = enum_reprs.iter().fold(iam_b, |b, e| {
    b.enum_attribute(
      format!("{}.{}", BASE_PACKAGE, e),
      "#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]",
    )
  });
  iam_b = enums.iter().fold(iam_b, |b, e| {
    b.type_attribute(format!("{}.{}", BASE_PACKAGE, e), "#[derive(serde::Serialize, serde::Deserialize)]")
  });
  // builder = messages.iter().fold(builder, |b, m| b.message_attribute(format!("{}.{}", BASE_PACKAGE, m), MESSAGE_ATTR));
  // iam_b = field_messages
  //   .iter()
  //   .fold(iam_b, |b, m| b.message_attribute(format!("{}.{}", BASE_PACKAGE, m), MODQL_MESSAGE_ATTR));

  iam_b.compile(&["proto/fusion_scheduler_api/v1/scheduler.proto"], &["proto"]).unwrap();
}
