mod _helper;
pub mod api;

use crate::application::ServerApplication;
use ultimate_web::Router;

pub fn routes() -> Router<ServerApplication> {
  Router::new().nest("/api", api::routes())
}
