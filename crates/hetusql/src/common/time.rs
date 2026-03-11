use chrono::{DateTime, FixedOffset, Local};

pub fn now_offset() -> DateTime<FixedOffset> {
  Local::now().fixed_offset()
}
