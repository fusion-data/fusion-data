use modql::filter::OpValInt64;
use prost_types::FieldMask;
use tonic::{Request, Response, Status};
use ultimate_grpc::utils::field_mask_match_with;

use crate::{
  ctx::CtxW,
  pb::fusion_iam::v1::{
    role_server::{Role, RoleServer},
    AssignRoleToPermissionsRequest, CreateRoleRequest, DeleteRoleRequest, DeleteRoleResponse, Empty, GetRoleRequest,
    PageRoleRequest, PageRoleResponse, RoleResponse, UpdateRoleRequest,
  },
  permission::{permission_serv, PermissionFilters},
  role::{role_serv, RoleFilters},
  util::grpc::{interceptor::auth_interceptor, GrpcServiceIntercepted},
};

use super::role_permission::RolePermissionFilter;

pub fn role_svc() -> GrpcServiceIntercepted<RoleServer<RoleService>> {
  RoleServer::with_interceptor(RoleService, auth_interceptor)
}

pub struct RoleService;
#[tonic::async_trait]
impl Role for RoleService {
  async fn create(&self, request: Request<CreateRoleRequest>) -> Result<Response<RoleResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let field_mask = request.field_mask.unwrap_or_default();
    let permission_ids = request.permission_ids;
    let entity_c = request.create_role.ok_or(Status::invalid_argument("create_role is required"))?;

    let id = role_serv::create(ctx, entity_c, permission_ids).await?;

    let resp = fetch_role_response(ctx, id, &field_mask).await?;
    Ok(Response::new(resp))
  }

  async fn get(&self, request: Request<GetRoleRequest>) -> Result<Response<RoleResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let field_mask = request.field_mask.unwrap_or_default();
    let id = request.id;

    let role = role_serv::find_by_id(ctx, id).await?;
    let permissions = if field_mask_match_with(&field_mask, "permissions") {
      let filters = PermissionFilters {
        role_perm_filter: RolePermissionFilter { role_id: Some(OpValInt64::Eq(id).into()), ..Default::default() },
        ..Default::default()
      };
      permission_serv::find_many(ctx, filters, None).await?.into_iter().map(Into::into).collect()
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

    role_serv::update_by_id(ctx, role_id, dto.try_into()?).await?;

    let resp = fetch_role_response(ctx, role_id, &field_mask).await?;
    Ok(Response::new(resp))
  }

  async fn delete(&self, request: Request<DeleteRoleRequest>) -> Result<Response<DeleteRoleResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    role_serv::delete_by_id(ctx, request.id).await?;
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

    role_serv::assign_permissions(ctx, role_id, permission_ids).await?;

    Ok(Response::new(Empty {}))
  }

  async fn page(&self, request: Request<PageRoleRequest>) -> Result<Response<PageRoleResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let filters = RoleFilters { filter: request.filter.into_iter().map(Into::into).collect(), ..Default::default() };

    let page = role_serv::page(ctx, filters, request.pagination.unwrap_or_default()).await?;
    Ok(Response::new(page.into()))
  }
}

async fn fetch_role_response(ctx: &CtxW, role_id: i64, field_mask: &FieldMask) -> Result<RoleResponse, Status> {
  let role = if field_mask_match_with(field_mask, "role") {
    let role = role_serv::find_by_id(ctx, role_id).await?;
    Some(role.into())
  } else {
    None
  };

  let permissions = if field_mask_match_with(field_mask, "permissions") {
    let filters = PermissionFilters {
      role_perm_filter: RolePermissionFilter { role_id: Some(OpValInt64::Eq(role_id).into()), ..Default::default() },
      ..Default::default()
    };
    permission_serv::find_many(ctx, filters, None).await?.into_iter().map(Into::into).collect()
  } else {
    vec![]
  };

  Ok(RoleResponse { role, permissions })
}
