use fusion_iam::start;

#[cfg(all(not(target_env = "msvc"), feature = "tikv-jemallocator"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> fusion_core::Result<()> {
  start::start_fusion_iam().await
}
