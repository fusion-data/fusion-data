//! HetuFlow 脚本包
//! 
//! 这个包包含部署和工具脚本，用于 HetuFlow 项目的管理和部署。

/// 脚本包版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 脚本包名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// JWE 密钥生成脚本路径
pub const JWE_KEY_GENERATOR_SCRIPT: &str = "generate-jwe-keys.sh";