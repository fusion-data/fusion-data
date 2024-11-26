use std::sync::Arc;

use fusiondata_context::ctx::{CtxW, RequestMetadata};
use tonic::{Request, Response, Status};
use ultimate::{application::Application, component::Component};

use crate::pb::fusion_iam::v1::{
  auth_server::{Auth, AuthServer},
  SigninRequest, SigninResponse,
};

use super::AuthSvc;

#[derive(Clone, Component)]
pub struct AuthRpc {
  #[component]
  auth_svc: AuthSvc,
}

impl AuthRpc {
  pub fn into_rpc(self) -> AuthServer<AuthRpc> {
    AuthServer::new(self)
  }
}

#[tonic::async_trait]
impl Auth for AuthRpc {
  #[tracing::instrument(skip(self, request))]
  async fn signin(&self, request: Request<SigninRequest>) -> Result<Response<SigninResponse>, Status> {
    let req_meta = RequestMetadata::from(request.metadata());
    let ctx = CtxW::new_with_req_meta(Application::global(), Arc::new(req_meta));
    // let trace_id = get_trace_id();
    // info!("trace_id: {:?}", trace_id);
    let res = self.auth_svc.signin(ctx, request.into_inner()).await?;
    Ok(Response::new(res))
  }
}
