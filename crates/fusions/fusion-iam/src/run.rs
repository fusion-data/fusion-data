use crate::endpoint::grpc::grpc_serve;

pub async fn run() -> ultimate::Result<()> {
  grpc_serve()?.await
}
