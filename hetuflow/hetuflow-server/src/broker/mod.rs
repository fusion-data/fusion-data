mod _broker;
mod broker_runner;
mod load_balancer;

use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

pub use _broker::Broker;
pub use broker_runner::LeaderOrFollowerRunner;
use load_balancer::LoadBalancer;

#[derive(Clone, Default)]
struct IsLeader(Arc<AtomicBool>);

impl IsLeader {
  pub fn leader(&self) -> bool {
    self.0.load(Ordering::SeqCst)
  }

  pub fn set_leader(&self) {
    if !self.0.swap(true, Ordering::SeqCst) {
      log::info!("Acquired leadership successfully, transitioning to leader mode");
    }
  }

  pub fn set_follower(&self) {
    if self.0.swap(false, Ordering::SeqCst) {
      log::info!("Transitioning to follower mode successfully");
    }
  }
}
