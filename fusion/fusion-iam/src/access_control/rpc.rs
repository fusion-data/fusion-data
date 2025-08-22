use tonic::{Request, Response, Status};
use log::debug;
use ultimate_core::component::Component;
use ultimate_grpc::GrpcServiceIntercepted;
use uuid::Uuid;

use crate::{
  pb::fusion_iam::v1::{
    CreatePolicyRequest, CreatePolicyResponse, DeletePolicyRequest, DeletePolicyResponse, GetPolicyRequest,
    GetPolicyResponse, UpdatePolicyRequest, UpdatePolicyResponse,
    access_control_server::{AccessControl, AccessControlServer},
  },
  util::grpc::interceptor::auth_interceptor,
};

use super::PolicySvc;

#[derive(Clone, Component)]
pub struct AccessControlRpc {
  #[component]
  policy_svc: PolicySvc,
}

impl AccessControlRpc {
  pub fn into_rpc(self) -> GrpcServiceIntercepted<AccessControlServer<AccessControlRpc>> {
    AccessControlServer::with_interceptor(self, auth_interceptor)
  }
}

#[tonic::async_trait]
impl AccessControl for AccessControlRpc {
  async fn create_policy_statement(
    &self,
    request: Request<CreatePolicyRequest>,
  ) -> Result<Response<CreatePolicyResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let policy = request.policy.parse().map_err(|_| Status::invalid_argument("Invalid policy structure"))?;

    let id = self.policy_svc.create(ctx, policy, request.description).await?;

    Ok(Response::new(CreatePolicyResponse { id: id.to_string(), policy_statement: None }))
  }

  async fn get_policy_statement(
    &self,
    request: Request<GetPolicyRequest>,
  ) -> Result<Response<GetPolicyResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let id = request.id.parse::<Uuid>().map_err(|_| Status::invalid_argument("Invalid policy statement id"))?;
    let policy_statement = self.policy_svc.find_by_id(ctx, id).await?;

    Ok(Response::new(GetPolicyResponse { policy_statement: Some(policy_statement.try_into()?) }))
  }

  async fn update_policy_statement(
    &self,
    request: Request<UpdatePolicyRequest>,
  ) -> Result<Response<UpdatePolicyResponse>, Status> {
    debug!("update_policy_statement: {:?}", request);
    todo!()
  }

  async fn delete_policy_statement(
    &self,
    request: Request<DeletePolicyRequest>,
  ) -> Result<Response<DeletePolicyResponse>, Status> {
    debug!("delete_policy_statement: {:?}", request);
    todo!()
  }
}
