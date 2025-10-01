//! modelsql::filter enables an expressive filtering language as described in [https://joql.org](https://joql.org).
//! It's serialization-agnostic but also provides JSON deserialization for convenience.

// -- Sub-Module
mod error;
#[cfg(feature = "with-sea-query")]
mod into_sea;
mod json;
pub(crate) mod nodes;
pub(crate) mod ops;
mod page;

// -- Re-Exports
pub use error::*;
pub use fusion_sql_macros::FilterNodes;
#[cfg(feature = "with-sea-query")]
pub use into_sea::*;
pub use nodes::group::*;
pub use nodes::node::*;
// pub use ops::op_val_array::*;
pub use ops::op_val_bool::*;
pub use ops::op_val_datetime::*;
pub use ops::op_val_nums::*;
pub use ops::op_val_string::*;
#[cfg(feature = "with-uuid")]
pub use ops::op_val_uuid::*;
pub use ops::op_val_value::*;
pub use ops::*;
pub use page::*;
