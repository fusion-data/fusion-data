//! ABAC 策略
pub mod policy;
mod utils;

pub use utils::{evaluate_condition, evaluate_policy};
