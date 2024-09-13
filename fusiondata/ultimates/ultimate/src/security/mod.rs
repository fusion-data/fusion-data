use serde::Deserialize;

mod error;
pub mod jose;
pub mod pwd;
mod security_utils;

pub use error::{Error, Result};
pub use security_utils::SecurityUtils;

#[derive(Deserialize)]
pub struct AccessToken {
  pub access_token: String,
}
