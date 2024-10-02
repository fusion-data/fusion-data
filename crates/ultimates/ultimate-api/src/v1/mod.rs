include!(concat!(env!("OUT_DIR"), "/ultimate_api.v1.rs"));

mod page;
mod ql;
mod r#type;

pub use page::*;
pub use ql::*;
pub use r#type::*;

pub use val_bool::OpBool;
pub use val_string::OpString;
