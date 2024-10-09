use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SchedulerConfig {
  pub node_id: &'static str,
  pub advertised_addr: Option<String>,
  pub namespaces: Vec<String>,
}
