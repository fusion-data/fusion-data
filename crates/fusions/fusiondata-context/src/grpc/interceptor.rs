use tonic::{Request, Status};

use crate::ctx::CtxW;

pub fn auth_interceptor(mut request: Request<()>) -> Result<Request<()>, Status> {
  let ctx: CtxW = request.metadata().try_into()?;
  request.extensions_mut().insert(ctx);

  Ok(request)
}
