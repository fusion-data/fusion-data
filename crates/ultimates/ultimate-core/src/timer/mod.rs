use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use hierarchical_hash_wheel_timer::{
  ClosureTimer, Timer as _,
  thread_timer::{self, TimerWithThread},
};
pub use hierarchical_hash_wheel_timer::{OneShotClosureState, PeriodicClosureState, TimerReturn};
use uuid::Uuid;

use crate::{DataError, application::ApplicationBuilder, plugin::Plugin};

pub type TimerCore = TimerWithThread<Uuid, OneShotClosureState<Uuid>, PeriodicClosureState<Uuid>>;

pub type ITimerRef = thread_timer::TimerRef<Uuid, OneShotClosureState<Uuid>, PeriodicClosureState<Uuid>>;

pub struct TimerRef(ITimerRef);
impl TimerRef {
  /// Schedule the `state` to be triggered once after the `timeout` expires
  ///
  /// # Note
  ///
  /// Depending on your system and the implementation used,
  /// there is always a certain lag between the triggering of the `state`
  /// and the `timeout` expiring on the system's clock.
  /// Thus it is only guaranteed that the `state` is not triggered *before*
  /// the `timeout` expires, but no bounds on the lag are given.
  pub fn schedule_once(&mut self, timeout: Duration, state: OneShotClosureState<Uuid>) {
    self.0.schedule_once(timeout, state);
  }

  /// Schedule the `state` to be triggered every `timeout` time units
  ///
  /// The first time, the `state` will be trigerreed after `delay` expires,
  /// and then again every `timeout` time units after, unless the
  /// [TimerReturn](TimerReturn) given specifies otherwise.
  ///
  /// # Note
  ///
  /// Depending on your system and the implementation used,
  /// there is always a certain lag between the triggering of the `state`
  /// and the `timeout` expiring on the system's clock.
  /// Thus it is only guaranteed that the `state` is not triggered *before*
  /// the `timeout` expires, but no bounds on the lag are given.
  pub fn schedule_periodic(&mut self, delay: Duration, period: Duration, state: PeriodicClosureState<Uuid>) {
    self.0.schedule_periodic(delay, period, state);
  }

  /// Schedule the `action` to be executed once after the `timeout` expires
  ///
  /// # Note
  ///
  /// Depending on your system and the implementation used,
  /// there is always a certain lag between the execution of the `action`
  /// and the `timeout` expiring on the system's clock.
  /// Thus it is only guaranteed that the `action` is not run *before*
  /// the `timeout` expires, but no bounds on the lag are given.
  pub fn schedule_action_once<F>(&mut self, id: Uuid, timeout: Duration, action: F)
  where
    F: FnOnce(Uuid) + Send + 'static,
  {
    self.0.schedule_action_once(id, timeout, action);
  }

  /// Schedule the `action` to be run every `timeout` time units
  ///
  /// The first time, the `action` will be run after `delay` expires,
  /// and then again every `timeout` time units after.
  ///
  /// # Note
  ///
  /// Depending on your system and the implementation used,
  /// there is always a certain lag between the execution of the `action`
  /// and the `timeout` expiring on the system's clock.
  /// Thus it is only guaranteed that the `action` is not run *before*
  /// the `timeout` expires, but no bounds on the lag are given.
  pub fn schedule_action_periodic<F>(&mut self, id: Uuid, delay: Duration, period: Duration, action: F)
  where
    F: FnMut(Uuid) -> TimerReturn<()> + Send + 'static,
  {
    self.0.schedule_action_periodic(id, delay, period, action);
  }

  /// Cancel the timer indicated by the unique `id`
  pub fn cancel(&mut self, id: &Uuid) {
    self.0.cancel(id);
  }
}

pub struct TimerPlugin;

#[async_trait]
impl Plugin for TimerPlugin {
  async fn build(&self, app: &mut ApplicationBuilder) {
    app.add_component(Timer::default());
  }
}

#[derive(Clone)]
pub struct Timer {
  timer_core: Arc<TimerCore>,
}

impl Timer {
  pub fn timer_ref(&self) -> TimerRef {
    TimerRef(self.timer_core.timer_ref())
  }

  pub fn shutdown_async(self) -> crate::Result<()> {
    self.timer_core.shutdown_async().map_err(|_e| DataError::server_error("Async stop timer error."))?;
    Ok(())
  }
}

impl Default for Timer {
  fn default() -> Self {
    Self { timer_core: Arc::new(TimerWithThread::for_uuid_closures()) }
  }
}
