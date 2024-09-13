// use std::{env, path::PathBuf};

fn main() {
  // println!("cargo::rerun-if-changed=proto/ultimate/");

  // let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  // prost_build::Config::new()
  //   .type_attribute("ultimate.v1.SortByDirection", r#"#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]"#)
  //   .file_descriptor_set_path(out_dir.join("ultimate_descriptor.bin"))
  //   .compile_protos(&["proto/ultimate/v1/ultimate.proto"], &["proto/ultimate"])
  //   .unwrap();
}
