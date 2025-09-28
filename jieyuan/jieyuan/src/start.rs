use fusion_core::{DataError, application::Application};
use fusion_db::DbPlugin;
use fusion_web::server::init_server;

use crate::endpoint::routes;

pub async fn start_jieyuan() -> Result<(), DataError> {
  Application::builder().add_plugin(DbPlugin).run().await?;
  let app = Application::global();

  tokio::spawn(init_server(routes().with_state(app)));

  Ok(())
}
