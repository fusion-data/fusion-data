pub mod config;
pub mod credential;
#[doc = include_str!("./expression/mod.md")]
pub mod expression;
pub mod metrics;
pub mod task;
pub mod types;
pub mod user;
pub mod utils;
pub mod workflow;
pub mod version {
  pub use semver::*;
}

#[macro_export]
macro_rules! generate_uuid_newtype {
  (
    $(Struct: $Struct:ty),* $(,)?
  ) => {
    $(
      impl $Struct {
        pub fn now_v7() -> Self {
          Self(uuid::Uuid::now_v7())
        }
        pub fn new_v4() -> Self {
          Self(uuid::Uuid::new_v4())
        }
      }
      impl std::str::FromStr for $Struct {
        type Err = uuid::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
          Ok(Self(uuid::Uuid::from_str(s)?))
        }
      }
    )*
  }
}
