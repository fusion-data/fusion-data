use fusiondata_context::app::get_app_state;
use tonic::{Request, Response, Status};
use tracing::info;
use ultimate::tracing::get_trace_id;

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
    let app = get_app_state();
    let trace_id = get_trace_id();
    info!("trace_id: {:?}", trace_id);
    let res = auth_serv::signin(app, request.into_inner()).await?;
    Ok(Response::new(res))
  }
}

pub fn auth_svc() -> AuthServer<AuthService> {
  AuthServer::new(AuthService)
}
