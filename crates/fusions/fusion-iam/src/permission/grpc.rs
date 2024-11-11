use fusiondata_context::ctx::CtxW;
use prost_types::FieldMask;
use tonic::{Request, Response, Status};
use ultimate_grpc::GrpcServiceIntercepted;

use crate::{
  pb::fusion_iam::v1::{
    permission_server::{Permission, PermissionServer},
    AssignPermmissionToRolesRequest, CreatePermissionRequest, DeletePermissionRequest, DeletePermissionResponse, Empty,
    GetPermissionRequest, PagePermissionRequest, PagePermissionResponse, PermissionDto, PermissionResponse,
    UpdatePermissionRequest,
  },
  util::grpc::interceptor::auth_interceptor,
};

use super::{permission_serv, PermissionFilters};

pub fn permission_svc() -> GrpcServiceIntercepted<PermissionServer<PermissionService>> {
  PermissionServer::with_interceptor(PermissionService, auth_interceptor)
}

pub struct PermissionService;

#[tonic::async_trait]
impl Permission for PermissionService {
  async fn create(&self, request: Request<CreatePermissionRequest>) -> Result<Response<PermissionResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let field_mask = request.field_mask.unwrap_or_default();
    let req = request.dto.ok_or(Status::invalid_argument("dto is required"))?.into();
    let id = permission_serv::create(ctx, req).await?;
    fetch_permission(ctx, field_mask, id).await
  }

  async fn update(&self, request: Request<UpdatePermissionRequest>) -> Result<Response<PermissionResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let field_mask = request.field_mask.unwrap_or_default();
    let req = request.dto.ok_or(Status::invalid_argument("dto is required"))?.into();

    permission_serv::update_by_id(ctx, request.id, req).await?;
    fetch_permission(ctx, field_mask, request.id).await
  }

  async fn delete(
    &self,
    request: Request<DeletePermissionRequest>,
  ) -> Result<Response<DeletePermissionResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    permission_serv::delete_by_id(ctx, request.id).await?;
    Ok(Response::new(DeletePermissionResponse {}))
  }

  async fn find(&self, request: Request<GetPermissionRequest>) -> Result<Response<PermissionDto>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let res = permission_serv::find_by_id(ctx, request.id).await?;
    Ok(Response::new(res.into()))
  }

  async fn page(&self, request: Request<PagePermissionRequest>) -> Result<Response<PagePermissionResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let filter =
      PermissionFilters { filter: request.filter.into_iter().map(|v| v.into()).collect(), ..Default::default() };

    let res = permission_serv::page(ctx, filter, request.pagination.unwrap_or_default()).await?;
    Ok(Response::new(res.into()))
  }

  async fn assign_role(&self, request: Request<AssignPermmissionToRolesRequest>) -> Result<Response<Empty>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    permission_serv::assign_roles(ctx, request.permission_id, request.role_ids).await?;
    Ok(Response::new(Empty {}))
  }
}

async fn fetch_permission(ctx: &CtxW, field_mask: FieldMask, id: i64) -> Result<Response<PermissionResponse>, Status> {
  let permission = if field_mask.paths.is_empty() {
    let permission = permission_serv::find_by_id(ctx, id).await?.into();
    Some(permission)
  } else {
    None
  };
  Ok(Response::new(PermissionResponse { id, permission }))
}
