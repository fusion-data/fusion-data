use std::sync::Arc;

use fusiondata_context::ctx::{CtxW, RequestMetadata};
use tonic::{Request, Response, Status};
use ultimate::application::Application;

use crate::pb::fusion_iam::v1::{
  auth_server::{Auth, AuthServer},
  SigninReplay, SigninRequest,
};

use super::auth_serv;

pub struct AuthService;

#[tonic::async_trait]
impl Auth for AuthService {
  #[tracing::instrument(skip(self, request))]
  async fn signin(&self, request: Request<SigninRequest>) -> Result<Response<SigninReplay>, Status> {
    let req_meta = RequestMetadata::from(request.metadata());
    let ctx = CtxW::new_with_req_meta(Application::global(), Arc::new(req_meta));
    // let trace_id = get_trace_id();
    // info!("trace_id: {:?}", trace_id);
    let res = auth_serv::signin(ctx, request.into_inner()).await?;
    Ok(Response::new(res))
  }
}

pub fn auth_svc() -> AuthServer<AuthService> {
  AuthServer::new(AuthService)
}
