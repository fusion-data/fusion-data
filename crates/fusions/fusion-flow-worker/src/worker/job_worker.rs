use std::time::Duration;

use fusion_flow_api::v1::{ListSchedulersRequest, scheduler_api_client::SchedulerApiClient};
use tonic::{
  Request,
  transport::{Channel, Endpoint, Uri},
};
use tracing::{debug, info};
use ultimate_core::{DataError, Result, component::Component, timer::Timer};

use crate::config::WorkerConfig;

/// 接收 TaskJob 并管理任务作业的执行
#[derive(Clone, Component)]
pub struct JobWorker {
  #[config]
  worker_config: WorkerConfig,

  #[component]
  _timer: Timer,
}

impl JobWorker {
  pub(crate) async fn run_loop(&self) -> Result<()> {
    let channel = Channel::balance_list(self.endpoint_seeds()?.into_iter());

    let token = self.worker_config.token.clone();
    let mut client = SchedulerApiClient::with_interceptor(channel, move |mut req: Request<()>| {
      req
        .metadata_mut()
        .append("authorization", format!("Bearer {}", token).parse().expect("Invalid Bearer token"));
      Ok(req)
    });

    // - 获取 scheduler nodes
    // - 连接到每个 scheduler node
    // - 在每一个 scheduler node 上 等待 job task

    let schedulers = client.list_schedulers(ListSchedulersRequest::default()).await?.into_inner().schedulers;
    debug!("Listed schedulers: {:?}", schedulers);

    loop {
      // TODO
      info!("Execute worker loop");
      tokio::time::sleep(Duration::from_secs(60)).await;
    }
  }

  fn endpoint_seeds(&self) -> Result<Vec<Endpoint>> {
    let mut seeds: Vec<Endpoint> = vec![];
    for seed in self.worker_config.node_seeds.iter() {
      let uri: Uri =
        seed.parse().map_err(|e| DataError::bad_request(format!("Invalid url [{}], error: {}", seed, e)))?;
      debug!("endpoint uri: {:?}", uri);
      seeds.push(uri.into());
    }
    Ok(seeds)
  }
}
