use fusion_common::time::{DateTime, FixedOffset, Local, deser::deserialize_fixed_offset, ser::serialize_fixed_offset};
use serde::{Deserialize, Serialize};

use crate::RunMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSetting {
  run_mode: RunMode,

  /// 应用名称。通常用于服务注册、日志打印等
  name: String,

  /// 系统使用时区，默认为 +08:00
  #[serde(serialize_with = "serialize_fixed_offset", deserialize_with = "deserialize_fixed_offset")]
  time_offset: FixedOffset,
}

impl AppSetting {
  pub fn run_mode(&self) -> &RunMode {
    &self.run_mode
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn time_offset(&self) -> &FixedOffset {
    &self.time_offset
  }

  pub fn time_now(&self) -> DateTime<FixedOffset> {
    Local::now().with_timezone(self.time_offset())
  }
}

#[cfg(test)]
mod tests {
  use config::{Config, File, FileFormat};
  use fusion_common::time::FixedOffset;

  use crate::RunMode;

  use super::AppSetting;

  #[test]
  fn test_app_conf() {
    let text = r#"
      run_mode = "dev"
      name = "fusion"
      time_offset = "+08:00"
    "#;
    let s = File::from_str(text, FileFormat::Toml);
    let c = Config::builder().add_source(s).build().unwrap();
    let app: AppSetting = c.try_deserialize().unwrap();
    println!("{:?}", app);
    assert_eq!(app.run_mode(), &RunMode::DEV);
    assert_eq!(app.time_offset(), &FixedOffset::east_opt(8 * 3600).unwrap());
  }
}
