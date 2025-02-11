// use std::time::Duration;

tonic::include_proto!("fusion_flow_api.v1");

#[cfg(feature = "with-db")]
mod db_helpers;

// impl SimpleSchedule {
//   pub fn interval_duration(&self) -> Result<Duration, String> {
//     duration_str::parse(&self.interval)
//   }

//   pub fn first_delay_duration(&self) -> Result<Duration, String> {
//     duration_str::parse(&self.first_delay)
//   }
// }
