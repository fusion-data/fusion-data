use fusion_scheduler::start::fusion_scheduler_start;

#[tokio::main]
async fn main() -> ultimate::Result<()> {
  fusion_scheduler_start().await?;
  Ok(())
}
