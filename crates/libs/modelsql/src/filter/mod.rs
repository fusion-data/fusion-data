//! modelsql::filter enables an expressive filtering language as described in [https://joql.org](https://joql.org).
//! It's serialization-agnostic but also provides JSON deserialization for convenience.

// -- Sub-Module
mod error;
mod into_sea;
mod json;
mod list_options;
pub(crate) mod nodes;
pub(crate) mod ops;

// -- Re-Exports
pub use error::*;
pub use into_sea::*;
pub use list_options::*;
pub use nodes::group::*;
pub use nodes::node::*;
pub use ops::op_val_bool::*;
pub use ops::op_val_nums::*;
pub use ops::op_val_string::*;

pub use modelsql_macros::FilterNodes;
#[cfg(feature = "with-uuid")]
pub use ops::op_val_uuid::*;
pub use ops::op_val_value::*;
pub use ops::*;
