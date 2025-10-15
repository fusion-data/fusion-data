/* 函数级注释：auth 模块入口，导出权限常量与枚举 */
pub mod permissions;
#[cfg(feature = "with-web")]
pub mod permission_middleware;