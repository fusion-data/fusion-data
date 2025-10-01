use axum::extract::FromRequestParts;
use fusion_core::{
  DataError,
  application::Application,
  security::pwd::{generate_pwd, verify_pwd},
};
use fusion_web::WebError;
use fusionsql::{ModelManager, filter::OpValString};
use hetumind_context::utils::make_token;
use hetumind_core::credential::TokenType;
use http::request::Parts;
use log::info;

use crate::domain::user::{UserBmc, UserFilter, UserForCreate, UserStatus};

use super::{SigninRequest, SigninResponse, SignupRequest};

#[derive(Clone)]
pub struct SignSvc {
  mm: ModelManager,
  application: Application,
}

impl SignSvc {
  pub async fn signin(&self, signin_req: SigninRequest) -> Result<SigninResponse, DataError> {
    let email = signin_req.as_email();
    let phone = signin_req.as_phone();

    let filter = if let Some(email) = email {
      UserFilter { email: Some(OpValString::eq(email)), ..Default::default() }
    } else if let Some(phone) = phone {
      UserFilter { phone: Some(OpValString::eq(phone)), ..Default::default() }
    } else {
      return Err(DataError::unauthorized("Parameter account must be email or phone"));
    };

    let user = UserBmc::find_unique(&self.mm, vec![filter])
      .await?
      .ok_or(DataError::unauthorized("User not found"))?;

    let password = user.password.ok_or_else(|| DataError::unauthorized("User password not set"))?;
    verify_pwd(&signin_req.password, &password).await?;

    let token = make_token(user.id.to_string(), self.application.fusion_config().security().pwd())?;
    Ok(SigninResponse { token, token_type: TokenType::Bearer })
  }

  pub async fn signup(&self, signup_req: SignupRequest) -> Result<(), DataError> {
    let password = generate_pwd(&signup_req.password).await?;
    let entity_c =
      UserForCreate { email: signup_req.email, phone: None, name: None, password, status: UserStatus::Enabled };

    let user_id = UserBmc::create(&self.mm, entity_c).await?;
    info!("User signup success: {}", user_id);
    Ok(())
  }
}

impl FromRequestParts<Application> for SignSvc {
  type Rejection = WebError;

  async fn from_request_parts(_parts: &mut Parts, state: &Application) -> Result<Self, Self::Rejection> {
    Ok(SignSvc { mm: state.component(), application: state.clone() })
  }
}
