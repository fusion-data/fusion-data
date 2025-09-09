use tonic::{Request, Status};

use fusiondata_context::ctx::CtxW;

#[allow(clippy::result_large_err)]
pub fn auth_interceptor(mut request: Request<()>) -> Result<Request<()>, Status> {
  let ctx: CtxW = CtxW::try_from(request.metadata())?;
  request.extensions_mut().insert(ctx);

  Ok(request)
}
