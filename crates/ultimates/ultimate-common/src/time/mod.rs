pub mod deser;
pub mod ser;

use std::sync::OnceLock;

pub use chrono::{DateTime, Duration, FixedOffset, Local, NaiveDate, NaiveTime, Offset, TimeDelta, TimeZone, Utc};

use super::Result;

pub type OffsetDateTime = DateTime<FixedOffset>;
pub type UtcDateTime = DateTime<Utc>;
pub type LocalDateTime = DateTime<Local>;

static LOCAL_OFFSET: OnceLock<FixedOffset> = OnceLock::new();

pub fn local_offset() -> &'static FixedOffset {
  LOCAL_OFFSET.get_or_init(_local_offset)
}

fn _local_offset() -> FixedOffset {
  Local::now().offset().fix()
}

#[inline]
pub fn now_utc() -> UtcDateTime {
  Utc::now()
}

#[inline]
pub fn now_offset() -> OffsetDateTime {
  Local::now().with_timezone(local_offset())
}

#[inline]
pub fn now() -> OffsetDateTime {
  now_offset()
}

pub fn now_epoch_millis() -> i64 {
  let now = now_utc();
  now.timestamp_millis()
}

#[inline]
pub fn now_epoch_seconds() -> i64 {
  now_utc().timestamp()
}

pub fn to_local<Tz: TimeZone>(t: DateTime<Tz>) -> DateTime<FixedOffset> {
  t.with_timezone(local_offset())
}

/// Returns an RFC 3339 and ISO 8601 date and time string such as 1996-12-19T16:39:57-08:00.
pub fn format_time<Tz: TimeZone>(time: DateTime<Tz>) -> Result<String> {
  Ok(time.to_rfc3339())
}

pub fn now_utc_plus_sec_str(sec: u64) -> Result<String> {
  let new_time = now_utc() + Duration::seconds(sec as i64);
  format_time(new_time)
}

pub fn utc_from_millis(milliseconds: i64) -> DateTime<Utc> {
  DateTime::<Utc>::MIN_UTC + Duration::milliseconds(milliseconds)
}

pub fn datetime_from_millis(milliseconds: i64) -> DateTime<FixedOffset> {
  utc_from_millis(milliseconds).with_timezone(local_offset())
}

pub fn parse_utc(moment: &str) -> Result<UtcDateTime> {
  let time = moment.parse::<UtcDateTime>()?;
  Ok(time)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_convert_std() {
    let now_utc = now_utc();
    println!("now utc is:\t{}", now_utc);

    let now_offset = now_offset();
    println!("now offset is:\t{}", now_offset);

    let now = now();
    println!("now is:\t\t{}", now);

    let offset_datetime: LocalDateTime = "2025-06-04T03:58:19.117041+00:00".parse().unwrap();
    println!("Offset DateTime is {}", offset_datetime);
  }
}
