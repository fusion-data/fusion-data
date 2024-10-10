mod application;
pub mod common;
mod ctx;
mod endpoint;
mod master;
pub mod pb;
mod service;
pub mod start;
mod worker;

pub static NODE_ALIVE_TIMEOUT_SECONDS: u64 = 30;
