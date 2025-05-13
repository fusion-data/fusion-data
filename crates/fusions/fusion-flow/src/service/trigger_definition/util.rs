use chrono::{DateTime, Local, Utc};
use chrono_tz::Tz;
use croner::Cron;
use ultimate_common::time::UtcDateTime;
use ultimate_core::{DataError, Result};

/// 计算 cron 表达式并返回下一个触发时间。表达式格式为 `* * * * * *`，对应：`秒 分 时 日 月 周`
///
/// # Parameters
///   - cron: cron 表达式
///   - tz: 时区，如：Asia/Chongqing
///   - now: 当前时间
///
/// # Returns
///   - 成功返回下次触发时间
///
pub fn cron_to_next_occurrence(cron: &str, tz: Option<&str>, now: &DateTime<Utc>) -> Result<UtcDateTime> {
  let mut cron = Cron::new(cron);
  cron.with_seconds_required();

  let next_occurrence = if let Some(tz) = tz {
    let tz: Tz = tz.parse().map_err(|_| DataError::bad_request(format!("Invalid Tz Format: {}", tz)))?;
    cron.find_next_occurrence(&now.with_timezone(&tz), false).map(|d| d.to_utc())
  } else {
    cron.find_next_occurrence(&Local::now(), false).map(|d| d.to_utc())
  };

  next_occurrence
    .map_err(|e| DataError::bad_request(format!("Get next occurrence time error from parse cron express: {}", e)))
}
