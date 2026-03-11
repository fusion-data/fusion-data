use hetus::common::ctx::Ctx;
use jieyuan_core::CtxExt;

/// 重新导出 hetu-common 的 Ctx 作为主要的上下文类型
pub type CtxPayload = Ctx;
