use std::{
  env::{self, VarError},
  path::PathBuf,
};

pub static CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub type Result<T> = core::result::Result<T, VarError>;

#[inline]
pub fn cargo_manifest_dir() -> Result<PathBuf> {
  from_env("CARGO_MANIFEST_DIR").map(PathBuf::from)
}

#[inline]
pub fn cargo_pkg_name() -> Result<String> {
  from_env("CARGO_PKG_NAME")
}

#[inline]
pub fn cargo_pkg_version() -> Result<String> {
  from_env("CARGO_PKG_VERSION")
}

#[inline]
fn from_env(name: &str) -> Result<String> {
  env::var(name)
}
