use fusion_flow_worker::start::fusion_flow_worker_start;

#[tokio::main]
async fn main() -> ultimate_core::Result<()> {
  fusion_flow_worker_start().await
}
