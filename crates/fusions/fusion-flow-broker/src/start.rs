use ultimate::{
  application::{Application, ApplicationBuilder},
  utils::handle_join_error,
};

use crate::Broker;

pub struct FusionFlowBroker {}

impl FusionFlowBroker {
  pub async fn init(_app_builder: &mut ApplicationBuilder) -> ultimate::Result<Self> {
    Ok(Self {})
  }

  pub async fn start(self) -> ultimate::Result<()> {
    let app = Application::global();
    let (master_handle, scheduler_handle) = app.component::<Broker>().spawn_loop().await?;

    let (master_ret, scheduler_ret) = tokio::join!(master_handle, scheduler_handle);

    handle_join_error(scheduler_ret, "scheduler");
    handle_join_error(master_ret, "master");

    Ok(())
  }
}
