use fusion_common::{ahash::HashMap, time::now_epoch_millis};
use hetuflow_core::protocol::ProcessInfo;
use mea::rwlock::RwLock;
use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};
use utils::*;
use uuid::Uuid;

mod cleanup_runner;
mod process_manager;
mod utils;

pub use cleanup_runner::*;
pub use process_manager::ProcessManager;

struct ProcessItem {
  pub info: ProcessInfo,
  // pub child: Arc<RwLock<Child>>,
}

#[derive(Clone, Default)]
struct ActiveProcesses(Arc<RwLock<HashMap<Uuid, ProcessItem>>>);

impl Deref for ActiveProcesses {
  type Target = Arc<RwLock<HashMap<Uuid, ProcessItem>>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
