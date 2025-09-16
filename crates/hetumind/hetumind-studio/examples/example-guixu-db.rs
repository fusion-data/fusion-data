use fusion_core::{DataError, application::Application};
use fusion_db::DbPlugin;
use hetumind::domain::user::{UserBmc, UserFilter};
use hetumind_context::ctx::CtxW;
use modelsql::filter::OpValString;

#[tokio::main]
async fn main() -> Result<(), DataError> {
  let application = Application::builder().add_plugin(DbPlugin).build().await?;
  let ctx = CtxW::new_super_admin(application.component());

  // let user_for_create = UserForCreate {
  //   email: "test@test.com".to_string(),
  //   phone: None,
  //   name: Some("test".to_string()),
  //   password: "test".to_string(),
  //   status: UserStatus::Active,
  // };

  // let uid = UserBmc::create(&app.db, user_for_create).await?;

  // let user_entity = UserBmc::get_by_id(&app.db, uid).await?;
  // println!("user_entity: {:?}", user_entity);

  let filter = UserFilter { email: Some(OpValString::EndsWith("@test.com".to_string()).into()), ..Default::default() };
  let users = UserBmc::find_many(ctx.mm(), vec![filter], None).await?;
  println!("users: {:?}", users);

  Ok(())
}
