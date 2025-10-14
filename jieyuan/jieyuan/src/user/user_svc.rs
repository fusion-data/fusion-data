use axum::extract::FromRequestParts;
use fusion_common::regex;
use fusion_core::{DataError, Result, application::Application, security::pwd::generate_pwd};
use fusion_web::WebError;
use fusionsql::{ModelManager, filter::OpValInt64, page::PageResult};

use jieyuan_core::model::{
  User, UserCredential, UserCredentialForInsert, UserFilter, UserForCreate, UserForPage, UserForUpdate,
  UserRoleForCreate,
};

use crate::utils::model_manager_from_parts;

use super::{UserBmc, UserCredentialBmc, UserRoleBmc};

#[derive(Clone)]
pub struct UserSvc {
  mm: ModelManager,
}

impl UserSvc {
  /// 创建新的用户服务实例
  pub fn new(mm: ModelManager) -> Self {
    Self { mm }
  }

  pub async fn create(&self, input: UserForCreate) -> Result<i64> {
    let mm = self.mm.get_txn_clone();
    mm.dbx().begin_txn().await?;

    let encrypted_pwd = if let Some(password) = input.password.as_deref() {
      if password.len() < 6 {
        return Err(DataError::bad_request("Password length cannot be less than 6 characters."));
      }
      generate_pwd(password).await?
    } else {
      let setting = Application::global().fusion_config();
      generate_pwd(setting.security().pwd().default_pwd()).await?
    };

    let id = UserBmc::create(&mm, Self::validate_and_init(input)?).await?;
    UserCredentialBmc::insert(&mm, UserCredentialForInsert { id, encrypted_pwd }).await?;

    mm.dbx().commit_txn().await?;
    Ok(id)
  }

  pub async fn page(&self, req: UserForPage) -> Result<PageResult<User>> {
    let page = UserBmc::page(&self.mm, req.filter, req.page).await?;
    Ok(page)
  }

  pub async fn find_option_by_id(&self, id: i64) -> Result<Option<User>> {
    let f = UserFilter { id: Some(OpValInt64::eq(id)), ..Default::default() };
    let u = UserBmc::find_unique(&self.mm, vec![f]).await?;
    Ok(u)
  }

  pub async fn find_by_id(&self, id: i64) -> Result<User> {
    let u = UserBmc::find_by_id(&self.mm, id).await?;
    Ok(u)
  }

  pub async fn update_by_id(&self, id: i64, req: UserForUpdate) -> Result<()> {
    UserBmc::update_by_id(&self.mm, id, req).await?;
    Ok(())
  }

  pub async fn delete_by_id(&self, id: i64) -> Result<()> {
    UserBmc::delete_by_id(&self.mm, id).await?;
    Ok(())
  }

  pub async fn get_fetch_credential(&self, req: UserFilter) -> Result<(User, UserCredential)> {
    let u = UserBmc::find_unique(&self.mm, vec![req])
      .await?
      .ok_or_else(|| DataError::not_found("User not exists."))?;
    let uc = UserCredentialBmc::find_by_id(&self.mm, u.id).await?;
    Ok((u, uc))
  }

  pub async fn assign_role(&self, user_id: i64, role_ids: Vec<i64>) -> Result<()> {
    let user_roles = role_ids.into_iter().map(|role_id| UserRoleForCreate { user_id, role_id }).collect();
    UserRoleBmc::insert_many(&self.mm, user_roles).await?;
    Ok(())
  }

  /// 校验数据并进行初始化。`email` 或 `phone` 至少有一个，若两个值都设置，则只有 `email` 有效。
  ///
  /// 当 `name` 未设置时，将从 `email` 或 `phone` 中取值。
  pub fn validate_and_init(mut input: UserForCreate) -> Result<UserForCreate> {
    if let Some(email) = input.email.as_deref() {
      if !regex::is_email(email) {
        return Err(DataError::bad_request("The 'email' field is invalid"));
      }
    } else if let Some(phone) = input.phone.as_deref() {
      if !regex::is_phone(phone) {
        return Err(DataError::bad_request("The 'phone' field is invalid"));
      }
    } else {
      return Err(DataError::bad_request("At least one 'email' or 'phone' is required"));
    };

    if input.name.as_deref().is_none_or(|n| n.is_empty()) {
      input.name = match input.email.as_deref() {
        Some(email) => email.split('@').next().map(ToString::to_string),
        None => input.phone.clone(),
      };
    }

    Ok(input)
  }
}

impl FromRequestParts<Application> for UserSvc {
  type Rejection = WebError;

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &Application,
  ) -> core::result::Result<Self, Self::Rejection> {
    let mm = model_manager_from_parts(parts, state)?;
    Ok(Self::new(mm))
  }
}
