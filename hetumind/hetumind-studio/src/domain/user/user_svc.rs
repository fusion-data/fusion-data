use axum::{extract::FromRequestParts, http::request::Parts};
use fusion_core::{
  DataError,
  application::Application,
  security::pwd::{generate_pwd, verify_pwd},
};
use fusion_web::WebError;
use fusionsql::{ModelManager, page::PageResult};
use hetumind_context::utils::get_mm_from_parts;

use crate::domain::user::UserForPage;

use super::{UserBmc, UserEntity, UserForCreate, UserForUpdate, UserForUpdatePassword};

pub struct UserSvc {
  mm: ModelManager,
}

impl UserSvc {
  pub async fn find_page(&self, for_query: UserForPage) -> Result<PageResult<UserEntity>, DataError> {
    UserBmc::page(&self.mm, vec![for_query.filter], for_query.page).await.map_err(DataError::from)
  }

  pub async fn create(&self, entity_c: UserForCreate) -> Result<i64, DataError> {
    let password = generate_pwd(&entity_c.password).await?;
    let entity_c = UserForCreate { password, ..entity_c };
    UserBmc::create(&self.mm, entity_c).await.map_err(DataError::from)
  }

  pub async fn update(&self, id: i64, entity_u: UserForUpdate) -> Result<(), DataError> {
    UserBmc::update_by_id(&self.mm, id, entity_u).await.map_err(DataError::from)
  }

  pub async fn update_password(&self, id: i64, entity_u: UserForUpdatePassword) -> Result<(), DataError> {
    let user = UserBmc::find_by_id(&self.mm, id).await.map_err(DataError::from)?;

    if let Some(old_password) = entity_u.old_password.as_deref() {
      let _pwd_ver = verify_pwd(old_password, user.password.as_deref().unwrap()).await?;
    } else if let Some(_code) = entity_u.code.as_deref() {
      // TODO 验证 code
    } else {
      // TODO 验证当前用户是否是管理员
    }

    let password = generate_pwd(&entity_u.password).await?;
    UserBmc::set_new_password(&self.mm, id, password).await.map_err(DataError::from)
  }

  pub async fn delete(&self, id: i64) -> Result<(), DataError> {
    UserBmc::delete_by_id(&self.mm, id).await.map_err(DataError::from)
  }

  pub async fn get_by_id(&self, id: i64) -> Result<Option<UserEntity>, DataError> {
    UserBmc::get_by_id(&self.mm, id).await.map_err(DataError::from)
  }

  pub async fn find_by_id(&self, id: i64) -> Result<UserEntity, DataError> {
    UserBmc::find_by_id(&self.mm, id).await.map_err(DataError::from)
  }
}

impl UserSvc {
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }
}

impl FromRequestParts<Application> for UserSvc {
  type Rejection = WebError;

  async fn from_request_parts(parts: &mut Parts, state: &Application) -> Result<Self, Self::Rejection> {
    get_mm_from_parts(parts, state).map(Self::new)
  }
}
