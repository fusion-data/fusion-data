use fusions::core::{DataError, application::Application};
use fusions::db::DbPlugin;
use fusions::web::server::WebServerBuilder;

use crate::endpoint::routes;

pub async fn start_jieyuan() -> Result<(), DataError> {
  Application::builder().add_plugin(DbPlugin).run().await?;
  let app = Application::global();

  let web_fut = WebServerBuilder::new(routes().with_state(app)).build();
  tokio::spawn(web_fut);

  Ok(())
}
