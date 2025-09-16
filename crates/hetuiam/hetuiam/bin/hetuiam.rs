use hetuiam::start;

#[tokio::main]
async fn main() -> fusion_core::Result<()> {
  start::start_fusion_iam().await
}
