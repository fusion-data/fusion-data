include!(concat!(env!("OUT_DIR"), "/ultimate_api.v1.rs"));

mod page;
mod ql;
mod r#type;

pub use page::*;
pub use ql::*;
pub use r#type::*;

pub use filter_string::OpString;
