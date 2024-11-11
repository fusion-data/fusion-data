use std::{env, path::PathBuf};

// static BASE_PACKAGE: &str = ".fusion_scheduler.v1";

// static MESSAGE_ATTR: &str = "#[derive(serde::Serialize, serde::Deserialize)]";
// static MODQL_MESSAGE_ATTR: &str = "#[derive(ultimate_db::modql::field::Fields)]";

fn main() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  let mut b = tonic_build::configure().emit_rerun_if_changed(true);

  b = b.file_descriptor_set_path(out_dir.join("fusion_scheduler_descriptor.bin"))
    // .message_attribute(BASE_PACKAGE, MESSAGE_ATTR)
    // .bytes(["."])
    .extern_path(".ultimate_api.v1", "::ultimate_api::v1")
    .extern_path(".fusion_scheduler_api.v1", "::fusion_scheduler_api::v1");

  // let modql_messages = ["CreateProcessRequest", "CreateTriggerRequest"];
  // b = modql_messages.iter().fold(b, |b, m| b.message_attribute(format!("{}.{}", BASE_PACKAGE, m), MODQL_MESSAGE_ATTR));

  b.compile_protos(
    &["proto/fusion_scheduler/v1/scheduler.proto"],
    &["proto", "../fusion-scheduler-api/proto", "../../ultimates/ultimate-api/proto"],
  )
  .unwrap();
}
