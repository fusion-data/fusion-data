use tonic::{Request, Response, Status};
use fusion_core::component::Component;
use fusion_grpc::GrpcServiceIntercepted;

use crate::{
  pb::fusion_iam::v1::{
    AssignUserToRolesRequest, CreateUserRequest, CreateUserResponse, DeleteUserResponse, Empty, FindUserRequest,
    PageUserRequest, PageUserResponse, UpdateUserRequest, UserDto, UserResponse, create_user_response,
    user_server::{User, UserServer},
  },
  util::grpc::interceptor::auth_interceptor,
};

use super::UserSvc;

#[derive(Clone, Component)]
pub struct UserRpc {
  #[component]
  user_svc: UserSvc,
}

impl UserRpc {
  pub fn into_rpc(self) -> GrpcServiceIntercepted<UserServer<UserRpc>> {
    UserServer::with_interceptor(self, auth_interceptor)
  }
}

#[tonic::async_trait]
impl User for UserRpc {
  async fn find(&self, request: Request<FindUserRequest>) -> Result<Response<UserResponse>, Status> {
    let ctx = request.extensions().try_into()?;
    let user = self.user_svc.find_option_by_id(ctx, request.get_ref().id).await?.map(UserDto::from);
    Ok(Response::new(UserResponse { user }))
  }

  async fn create(&self, request: Request<CreateUserRequest>) -> Result<Response<CreateUserResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let returining_payload = request.returining_payload;

    let ctx = (&exts).try_into()?;
    let id = self.user_svc.create(ctx, request.try_into()?).await?;

    let data = if returining_payload {
      let u = self.user_svc.find_by_id(ctx, id).await?;
      create_user_response::Data::User(u.into())
    } else {
      create_user_response::Data::Id(id)
    };
    Ok(Response::new(CreateUserResponse { data: Some(data) }))
  }

  async fn update(&self, request: Request<UpdateUserRequest>) -> Result<Response<UserResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;
    let id = request.id;
    let returning_payload = request.returning_payload;

    self.user_svc.update_by_id(ctx, id, request.try_into()?).await?;

    let user = if returning_payload {
      let u = self.user_svc.find_option_by_id(ctx, id).await?;
      u.map(UserDto::from)
    } else {
      None
    };
    Ok(Response::new(UserResponse { user }))
  }

  async fn page(&self, request: Request<PageUserRequest>) -> Result<Response<PageUserResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let page = self.user_svc.page(ctx, request.into()).await?;
    Ok(Response::new(page.into()))
  }

  async fn delete(&self, request: Request<FindUserRequest>) -> Result<Response<DeleteUserResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let id = request.id;
    self.user_svc.delete_by_id(ctx, id).await?;
    Ok(Response::new(DeleteUserResponse {}))
  }

  async fn assign_role(&self, request: Request<AssignUserToRolesRequest>) -> Result<Response<Empty>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let user_id = request.user_id;
    let role_ids = request.role_ids;

    self.user_svc.assign_role(ctx, user_id, role_ids).await?;
    Ok(Response::new(Empty {}))
  }
}
