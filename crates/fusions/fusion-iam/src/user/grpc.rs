use tonic::{Request, Response, Status};
use ultimate_grpc::GrpcServiceIntercepted;

use crate::{
  pb::fusion_iam::v1::{
    create_user_response,
    user_server::{User, UserServer},
    AssignUserToRolesRequest, CreateUserRequest, CreateUserResponse, DeleteUserResponse, Empty, FindUserRequest,
    PageUserRequest, PageUserResponse, UpdateUserRequest, UserDto, UserResponse,
  },
  util::grpc::interceptor::auth_interceptor,
};

use super::user_serv;

pub fn user_svc() -> GrpcServiceIntercepted<UserServer<UserService>> {
  UserServer::with_interceptor(UserService, auth_interceptor)
}

pub struct UserService;

#[tonic::async_trait]
impl User for UserService {
  async fn find(&self, request: Request<FindUserRequest>) -> Result<Response<UserResponse>, Status> {
    let ctx = request.extensions().try_into()?;
    let user = user_serv::find_option_by_id(ctx, request.get_ref().id).await?.map(UserDto::from);
    Ok(Response::new(UserResponse { user }))
  }

  async fn create(&self, request: Request<CreateUserRequest>) -> Result<Response<CreateUserResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let returining_payload = request.returining_payload;

    let ctx = (&exts).try_into()?;
    let id = user_serv::create(ctx, request.try_into()?).await?;

    let data = if returining_payload {
      let u = user_serv::find_by_id(ctx, id).await?;
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

    user_serv::update_by_id(ctx, id, request.try_into()?).await?;

    let user = if returning_payload {
      let u = user_serv::find_option_by_id(ctx, id).await?;
      u.map(UserDto::from)
    } else {
      None
    };
    Ok(Response::new(UserResponse { user }))
  }

  async fn page(&self, request: Request<PageUserRequest>) -> Result<Response<PageUserResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let page = user_serv::page(ctx, request.into()).await?;
    Ok(Response::new(page.into()))
  }

  async fn delete(&self, request: Request<FindUserRequest>) -> Result<Response<DeleteUserResponse>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let id = request.id;
    user_serv::delete_by_id(ctx, id).await?;
    Ok(Response::new(DeleteUserResponse {}))
  }

  async fn assign_role(&self, request: Request<AssignUserToRolesRequest>) -> Result<Response<Empty>, Status> {
    let (_, exts, request) = request.into_parts();
    let ctx = (&exts).try_into()?;

    let user_id = request.user_id;
    let role_ids = request.role_ids;

    user_serv::assign_role(ctx, user_id, role_ids).await?;
    Ok(Response::new(Empty {}))
  }
}
