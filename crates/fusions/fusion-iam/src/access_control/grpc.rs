use tonic::{Request, Response, Status};
use tracing::debug;
use ultimate_grpc::GrpcServiceIntercepted;
use uuid::Uuid;

use crate::{
  pb::fusion_iam::v1::{
    access_control_server::{AccessControl, AccessControlServer},
    CreatePolicyRequest, CreatePolicyResponse, DeletePolicyRequest, DeletePolicyResponse, GetPolicyRequest,
    GetPolicyResponse, UpdatePolicyRequest, UpdatePolicyResponse,
  },
  util::grpc::interceptor::auth_interceptor,
};

use super::policy_serv;

pub fn access_control_svc() -> GrpcServiceIntercepted<AccessControlServer<AccessControlService>> {
  AccessControlServer::with_interceptor(AccessControlService, auth_interceptor)
}

pub struct AccessControlService;

#[tonic::async_trait]
impl AccessControl for AccessControlService {
  async fn create_policy_statement(
    &self,
    request: Request<CreatePolicyRequest>,
  ) -> Result<Response<CreatePolicyResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let policy = request.policy.parse().map_err(|_| Status::invalid_argument("Invalid policy structure"))?;

    let id = policy_serv::create(ctx, policy, request.description).await?;

    Ok(Response::new(CreatePolicyResponse { id: id.to_string(), policy_statement: None }))
  }

  async fn get_policy_statement(
    &self,
    request: Request<GetPolicyRequest>,
  ) -> Result<Response<GetPolicyResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let id = request.id.parse::<Uuid>().map_err(|_| Status::invalid_argument("Invalid policy statement id"))?;
    let policy_statement = policy_serv::find_by_id(ctx, id).await?;

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
