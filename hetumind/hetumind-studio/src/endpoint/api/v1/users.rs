use axum::{
  Json,
  extract::Path,
  routing::{get, post},
};
use fusion_core::application::Application;
use fusion_web::{Router, WebResult, ok_json};
use fusionsql::page::PageResult;
use jieyuan_core::web::middleware::permission_layer;

use crate::domain::user::{UserEntity, UserForPage, UserForUpdate, UserSvc};

pub fn routes() -> Router<Application> {
  Router::new()
    .route(
      "/item/{id}",
      get(get_user_by_id)
        .layer(permission_layer(["user_read"]))
        .put(update_user_by_id)
        .layer(permission_layer(["user_update"])),
    )
    .route("/query", post(query_user).layer(permission_layer(["user_read"])))
}

async fn get_user_by_id(user_svc: UserSvc, Path(id): Path<i64>) -> WebResult<Option<UserEntity>> {
  let user = user_svc.get_by_id(id).await?;
  ok_json!(user)
}

async fn update_user_by_id(
  user_svc: UserSvc,
  Path(id): Path<i64>,
  Json(for_update): Json<UserForUpdate>,
) -> WebResult<()> {
  user_svc.update(id, for_update).await?;
  ok_json!()
}

async fn query_user(user_svc: UserSvc, Json(for_page): Json<UserForPage>) -> WebResult<PageResult<UserEntity>> {
  let paged = user_svc.find_page(for_page).await?;
  ok_json!(paged)
}
