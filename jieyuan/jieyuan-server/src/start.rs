use ultimates::core::logforth::LogforthPlugin;
use ultimates::core::{DataError, application::Application};
use ultimates::db::DbPlugin;
use ultimates::web::server::WebServerBuilder;

use crate::endpoint::routes;

pub async fn start_jieyuan() -> Result<(), DataError> {
  Application::builder().add_plugin(LogforthPlugin).add_plugin(DbPlugin).run().await?;
  let app = Application::global();

  let web_fut = WebServerBuilder::new(routes().with_state(app)).build();
  tokio::spawn(web_fut);

  Ok(())
}
