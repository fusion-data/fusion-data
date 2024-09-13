use fusion_iam::{app::get_app_state, run};
use tracing::info;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() -> ultimate::Result<()> {
  let ret = get_app_state().runtime().block_on(run());
  info!("Application run finished: {:?}", ret);
  Ok(())
}
