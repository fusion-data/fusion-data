pub mod hello_world {
  tonic::include_proto!("fusion_iam.v1");
}

///
/// ```
/// cargo run -p fruitbox-iam --example grpc-web --features tonic-web
/// ``
///
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  #[cfg(feature = "tonic-web")]
  test_grpc_web().await?;

  Ok(())
}

#[cfg(feature = "tonic-web")]
async fn test_grpc_web() -> Result<(), Box<dyn std::error::Error>> {
  use hello_world::{role_client::RoleClient, GetRoleRequest};
  use hyper_util::rt::TokioExecutor;
  use tonic_web::GrpcWebClientLayer;

  // Must use hyper directly...
  let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build_http();
  let svc_b = tower::ServiceBuilder::new().layer(GrpcWebClientLayer::new());

  let svc = svc_b.service(client);

  let mut client = RoleClient::with_origin(svc, "http://127.0.0.1:8889".try_into()?);

  let mut request = tonic::Request::new(GetRoleRequest { id: 1, ..Default::default() });
  request.metadata_mut().insert("authorization", "Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..pmcUdN9wb8J63fkU6JDOJw.kDRISHrRKvo58GSC1TVCNGmjfnojcWFgcuhfNypsQjI.kPWYQa3ApiP7QFkVLNWwrw".parse().unwrap());

  let response = client.get(request).await?;

  println!("RESPONSE={:?}", response);

  Ok(())
}
