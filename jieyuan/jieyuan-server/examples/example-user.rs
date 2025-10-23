use fusions::core::{DataError, application::Application};
use fusions::db::DbPlugin;
use fusionsql::filter::OpValInt64;

use jieyuan_core::model::{SigninRequest, UserFilter, UserForCreate};
use jieyuan_server::{access_control::AuthSvc, user::UserSvc};

#[tokio::main]
async fn main() -> Result<(), DataError> {
  let app = Application::builder().add_plugin(DbPlugin).build().await?;
  let user_svc = UserSvc::new(app.component());

  // Create User
  let input = UserForCreate {
    email: Some(String::from("guest@hetumind.com")),
    name: Some(String::from("访客")),
    password: Some(String::from("2025.Hetumind")),
    ..Default::default()
  };
  let user_id = user_svc.create(input).await?;

  // Login User
  let auth_svc = AuthSvc::new(user_svc.clone());
  let input = SigninRequest {
    email: Some(String::from("guest@hetumind.com")),
    password: String::from("2025.Hetumind"),
    ..Default::default()
  };
  let signin_response = auth_svc.signin(input).await?;
  println!("Signin response: {:?}", signin_response);

  // Get User
  let (user, user_credential) = user_svc
    .get_fetch_credential(UserFilter { id: Some(OpValInt64::eq(user_id)), ..Default::default() })
    .await?;
  println!("User: {:?}", user);
  println!("User Encrypted Password: {}", user_credential.encrypted_pwd);

  Ok(())
}
