include!(concat!(env!("OUT_DIR"), "/ultimate_api.v1.rs"));

mod page;
mod ql;
mod types;

pub use page::*;
// pub use ql::*;
// pub use types::*;

pub use val_bool::OpBool;
pub use val_string::OpString;
