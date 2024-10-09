use std::{env, path::PathBuf};

static BASE_PACKAGE: &str = ".ultimate_api.v1";

static SERDE_ATTR: &str = "#[derive(serde::Serialize, serde::Deserialize)]";
static SERDE_REPR_ATTR: &str = "#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]";
// static MODQL_MESSAGE_ATTR: &str = "#[derive(modql::field::Fields)]";

fn main() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  let mut c = prost_build::Config::new();
  c.out_dir(out_dir).message_attribute(BASE_PACKAGE, SERDE_ATTR);

  ["OpNumber", "ValString.OpString", "ValBool.OpBool"].into_iter().for_each(|en| {
    c.enum_attribute(format!("{}.{}", BASE_PACKAGE, en), SERDE_REPR_ATTR);
  });

  ["ValString.value", "ValBool.value", "ValDouble.value", "ValInt32.value", "ValInt64.value"].into_iter().for_each(
    |m| {
      c.type_attribute(format!("{}.{}", BASE_PACKAGE, m), SERDE_ATTR);
    },
  );

  // let modql_messages = ["CreateProcessDefinitionRequest", "CreateTriggerDefinitionRequest"];
  // b = modql_messages.iter().fold(b, |b, m| b.message_attribute(format!("{}.{}", BASE_PACKAGE, m), MODQL_MESSAGE_ATTR));

  c.compile_protos(
    &["proto/ultimate_api/v1/types.proto", "proto/ultimate_api/v1/page.proto", "proto/ultimate_api/v1/ql.proto"],
    &["proto"],
  )
  .unwrap();
}
