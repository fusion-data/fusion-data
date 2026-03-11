use uuid::Uuid;

use super::Error;

/// Generates a new UUID version 4.
///
/// UUID version 4 is randomly generated.
pub fn new_v4() -> Uuid {
  Uuid::new_v4()
}

/// Generates a new UUID version 7 with the now time.
///
/// UUID version 7 is a time-ordered UUID which is well-suited for use as a database key.
/// This function uses `Uuid::now_v7()` which is suitable for most cases.
pub fn now_v7() -> Uuid {
  Uuid::now_v7()
}

/// If `uuid` is a version-7 UUID, return the embedded timestamp as a `SystemTime`,
/// otherwise return `None`.
pub fn to_time_epoch_ms(uuid: &Uuid) -> Result<i64, Error> {
  // make sure this really is a v7 UUID
  if uuid.get_version_num() != 7 {
    return Err(Error::FailExtractTimeNoUuidV7(*uuid));
  }

  // as_u128() lays out the bytes big-endian, so the top 48 bits are our ms timestamp
  let as_int = uuid.as_u128();
  let ts_ms = (as_int >> 80) as i64; // drop the low 80 bits, leaving top 48

  // convert ms-since-epoch
  Ok(ts_ms)
}

#[cfg(test)]
mod tests {
  use std::time::SystemTime;

  use uuid::Version;

  use super::*;

  type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

  #[test]
  fn test_extra_uuid_new_v4_simple() -> Result<()> {
    // -- Setup & Fixtures
    // (no specific setup needed for this test)

    // -- Exec
    let uuid_v4 = new_v4();

    // -- Check
    assert_eq!(uuid_v4.get_version(), Some(Version::Random));
    Ok(())
  }

  #[test]
  fn test_extra_uuid_new_v7_simple() -> Result<()> {
    // -- Setup & Fixtures
    // (no specific setup needed for this test)

    // -- Exec
    let uuid_v7_1 = now_v7();
    // Ensure a slight delay for time-based ordering if testing monotonicity strictly,
    // but for basic version check, it's not critical.
    // For now, we just check the version.
    std::thread::sleep(std::time::Duration::from_micros(100)); // Ensure time moves for next v7
    let uuid_v7_2 = now_v7();

    // -- Check
    assert_eq!(uuid_v7_1.get_version(), Some(Version::SortRand));
    assert_eq!(uuid_v7_2.get_version(), Some(Version::SortRand));
    // V7 UUIDs generated sequentially should be ordered
    // Note: Uuid::now_v7() guarantees monotonicity for calls within the same millisecond
    // by incrementing a sequence counter. If calls are in different milliseconds,
    // the time component will ensure ordering.
    assert!(uuid_v7_1 < uuid_v7_2, "UUID {} should be less than UUID {}", uuid_v7_1, uuid_v7_2);

    Ok(())
  }

  #[test]
  fn test_extra_uuid_to_time_epoch_ms_ok() -> Result<()> {
    // -- Setup & Fixtures
    let t0_ms = system_time_to_ms(SystemTime::now())?;
    // Small delay to ensure time progresses if system clock resolution is low
    // relative to execution speed, and that now_v7() has a distinct timestamp.
    std::thread::sleep(std::time::Duration::from_millis(1));

    let uuid_v7 = now_v7(); // Uuid::now_v7() is used here

    std::thread::sleep(std::time::Duration::from_millis(1));
    let t1_ms = system_time_to_ms(SystemTime::now())?;

    // -- Exec
    let extracted_ts_ms = to_time_epoch_ms(&uuid_v7)?;

    // -- Check
    // The extracted timestamp should be between t0_ms and t1_ms (inclusive).
    assert!(
      extracted_ts_ms >= t0_ms,
      "Extracted timestamp {}ms should be greater than or equal to t0_ms {}ms. UUID: {}",
      extracted_ts_ms,
      t0_ms,
      uuid_v7
    );
    assert!(
      extracted_ts_ms <= t1_ms,
      "Extracted timestamp {}ms should be less than or equal to t1_ms {}ms. UUID: {}",
      extracted_ts_ms,
      t1_ms,
      uuid_v7
    );

    Ok(())
  }

  #[test]
  fn test_extra_uuid_to_time_epoch_ms_err_not_v7() -> Result<()> {
    // -- Setup & Fixtures
    let uuid_v4 = new_v4();

    // -- Exec
    let result = to_time_epoch_ms(&uuid_v4);

    // -- Check
    match result {
      Err(Error::FailExtractTimeNoUuidV7(id)) => {
        assert_eq!(id, uuid_v4, "The UUID in the error should match the input UUID.");
      }
      Ok(ts) => {
        return Err(
          format!("Expected FailExtractTimeNoUuidV7 error for non-v7 UUID, but got Ok({}). UUID was: {}", ts, uuid_v4)
            .into(),
        );
      }
      Err(other_error) => {
        return Err(
          format!(
            "Expected FailExtractTimeNoUuidV7 error, but got a different error: {:?}. UUID was: {}",
            other_error, uuid_v4
          )
          .into(),
        );
      }
    }

    Ok(())
  }

  // region:    --- Support
  fn system_time_to_ms(st: SystemTime) -> Result<i64> {
    Ok(
      st.duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get duration since UNIX_EPOCH: {}", e))?
        .as_millis() as i64,
    )
  }
  // endregion: --- Support
}
