use fusiondata_context::ctx::CtxW;
use modelsql::filter::OpValInt64;
use protobuf::FieldMask;
use tonic::{Request, Response, Status};
use ultimate_core::component::Component;
use ultimate_grpc::{GrpcServiceIntercepted, utils::field_mask_match_with};

use crate::{
  pb::fusion_iam::v1::{
    AssignRoleToPermissionsRequest, CreateRoleRequest, DeleteRoleRequest, DeleteRoleResponse, Empty, GetRoleRequest,
    PageRoleRequest, PageRoleResponse, RoleResponse, UpdateRoleRequest,
    role_server::{Role, RoleServer},
  },
  permission::{PermissionFilters, PermissionSvc},
  util::grpc::interceptor::auth_interceptor,
};

use super::{RoleFilters, RoleSvc, role_permission::RolePermissionFilter};

#[derive(Clone, Component)]
pub struct RoleRpc {
  #[component]
  role_svc: RoleSvc,
  #[component]
  permission_svc: PermissionSvc,
}

impl RoleRpc {
  pub fn into_rpc(self) -> GrpcServiceIntercepted<RoleServer<RoleRpc>> {
    RoleServer::with_interceptor(self, auth_interceptor)
  }

  async fn fetch_role_response(
    &self,
    ctx: &CtxW,
    role_id: i64,
    field_mask: &FieldMask,
  ) -> Result<RoleResponse, Status> {
    let role = if field_mask_match_with(field_mask, "role") {
      let role = self.role_svc.find_by_id(ctx, role_id).await?;
      Some(role.into())
    } else {
      None
    };

    let permissions = if field_mask_match_with(field_mask, "permissions") {
      let filters = PermissionFilters {
        role_perm_filter: RolePermissionFilter { role_id: Some(OpValInt64::Eq(role_id).into()), ..Default::default() },
        ..Default::default()
      };
      self.permission_svc.find_many(ctx, filters, None).await?.into_iter().map(Into::into).collect()
    } else {
      vec![]
    };

    Ok(RoleResponse { role, permissions })
  }
}

#[tonic::async_trait]
impl Role for RoleRpc {
  async fn create(&self, request: Request<CreateRoleRequest>) -> Result<Response<RoleResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let field_mask = request.field_mask.unwrap_or_default();
    let permission_ids = request.permission_ids;
    let entity_c = request.create_role.ok_or(Status::invalid_argument("create_role is required"))?;

    let id = self.role_svc.create(ctx, entity_c, permission_ids).await?;

    let resp = self.fetch_role_response(ctx, id, &field_mask).await?;
    Ok(Response::new(resp))
  }

  async fn get(&self, request: Request<GetRoleRequest>) -> Result<Response<RoleResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let field_mask = request.field_mask.unwrap_or_default();
    let id = request.id;

    let role = self.role_svc.find_by_id(ctx, id).await?;
    let permissions = if field_mask_match_with(&field_mask, "permissions") {
      let filters = PermissionFilters {
        role_perm_filter: RolePermissionFilter { role_id: Some(OpValInt64::Eq(id).into()), ..Default::default() },
        ..Default::default()
      };
      self.permission_svc.find_many(ctx, filters, None).await?.into_iter().map(Into::into).collect()
    } else {
      vec![]
    };

    Ok(Response::new(RoleResponse { role: Some(role.into()), permissions }))
  }

  async fn update(&self, request: Request<UpdateRoleRequest>) -> Result<Response<RoleResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let role_id = request.id;
    let field_mask = request.field_mask.unwrap_or_default();
    let dto = request.dto.ok_or(Status::invalid_argument("dto is required"))?;

    self.role_svc.update_by_id(ctx, role_id, dto.try_into()?).await?;

    let resp = self.fetch_role_response(ctx, role_id, &field_mask).await?;
    Ok(Response::new(resp))
  }

  async fn delete(&self, request: Request<DeleteRoleRequest>) -> Result<Response<DeleteRoleResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    self.role_svc.delete_by_id(ctx, request.id).await?;
    Ok(Response::new(DeleteRoleResponse {}))
  }

  async fn assign_permission(
    &self,
    request: Request<AssignRoleToPermissionsRequest>,
  ) -> Result<Response<Empty>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let role_id = request.role_id;
    let permission_ids = request.permission_ids;

    self.role_svc.assign_permissions(ctx, role_id, permission_ids).await?;

    Ok(Response::new(Empty {}))
  }

  async fn page(&self, request: Request<PageRoleRequest>) -> Result<Response<PageRoleResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let filters = RoleFilters { filter: request.filter.into_iter().map(Into::into).collect(), ..Default::default() };

    let page = self.role_svc.page(ctx, filters, request.pagination.unwrap_or_default()).await?;
    Ok(Response::new(page.into()))
  }
}
