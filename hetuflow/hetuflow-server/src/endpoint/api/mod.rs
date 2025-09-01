pub mod v1;

use fusion_web::Router;

use crate::application::ServerApplication;

pub fn routes() -> Router<ServerApplication> {
  Router::new().nest("/v1", v1::routes())
}
