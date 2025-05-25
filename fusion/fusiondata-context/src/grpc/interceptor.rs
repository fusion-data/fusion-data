use tonic::{Request, Status};

use crate::ctx::CtxW;

#[allow(clippy::result_large_err)]
pub fn auth_interceptor(mut request: Request<()>) -> Result<Request<()>, Status> {
  let ctx: CtxW = request.metadata().try_into()?;
  request.extensions_mut().insert(ctx);

  Ok(request)
}
