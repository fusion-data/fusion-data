///
/// ```
/// cargo run -p fruitbox-iam --example grpc-web --features tonic-web
/// ``
use fusion_iam::pb::fusion_iam::v1::{
  auth_client::AuthClient, user_client::UserClient, FilterUserRequest, PageUserRequest, SigninRequest, TokenKind,
};
use tonic::{transport::Channel, Request};
use ultimate_api::v1::{OpNumber, OpString, Pagination, ValInt32, ValString};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let channel = Channel::from_static("http://localhost:58010").connect().await?;

  let mut auth_client = AuthClient::new(channel.clone());
  let req = SigninRequest {
    email: Some("admin@fusiondata.com".to_string()),
    password: "2024.Fusiondata".to_string(),
    ..Default::default()
  };
  let resp = auth_client.signin(req).await?;
  println!("{:?}", resp);

  assert_eq!(TokenKind::Bearer, resp.get_ref().token_kind());

  let mut user_client = UserClient::with_interceptor(channel.clone(), |mut request: Request<()>| {
    request
      .metadata_mut()
      .insert("authorization", format!("Bearer {}", resp.get_ref().token).parse().unwrap());
    Ok(request)
  });
  let page_req = PageUserRequest {
    pagination: Some(Pagination::new_default()),
    filter: vec![FilterUserRequest {
      status: ValInt32::new_value(OpNumber::Eq, 100).into(),
      ctime: ValString::new_value(OpString::Gte, "2024-08-08T00:00:00+08:00").into(),
      ..Default::default()
    }],
  };
  println!("Page Req is: {:?}", page_req);
  let page_resp = user_client.page(page_req).await?;
  println!("{:?}", page_resp.get_ref());
  assert!(!page_resp.get_ref().items.is_empty());

  Ok(())
}
