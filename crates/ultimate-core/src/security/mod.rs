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

/// Recommended length of a salt: 16-bytes.
///
/// This recommendation comes from the [PHC string format specification]:
///
/// > The role of salts is to achieve uniqueness. A *random* salt is fine
/// > for that as long as its length is sufficient; a 16-byte salt would
/// > work well (by definition, UUID are very good salts, and they encode
/// > over exactly 16 bytes). 16 bytes encode as 22 characters in B64.
///
/// [PHC string format specification]: https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md#function-duties
pub const RECOMMENDED_LENGTH: usize = 16;

/// Error message used with `expect` for when internal invariants are violated
/// (i.e. the contents of a [`Salt`] should always be valid)
pub(crate) const INVARIANT_VIOLATED_MSG: &str = "salt string invariant violated";
