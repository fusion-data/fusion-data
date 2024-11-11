use fusion_scheduler::start::fusion_scheduler_worker_start;

#[tokio::main]
async fn main() -> ultimate::Result<()> {
  fusion_scheduler_worker_start().await
}
