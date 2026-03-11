use base64ct::{Base64, Base64Url, Base64UrlUnpadded, Encoding};
use uuid::Uuid;

use super::extra_uuid::{new_v4, now_v7};
use super::{Error, support};

// region:    --- v4

/// Generates a new UUID version 4 and encodes it using standard Base64.
pub fn new_v4_b64() -> String {
  let uuid = new_v4();
  Base64::encode_string(uuid.as_bytes())
}

/// Generates a new UUID version 4 and encodes it using URL-safe Base64.
pub fn new_v4_b64url() -> String {
  let uuid = new_v4();
  Base64Url::encode_string(uuid.as_bytes())
}

/// Generates a new UUID version 4 and encodes it using URL-safe Base64 without padding.
pub fn new_v4_b64url_nopad() -> String {
  let uuid = new_v4();
  Base64UrlUnpadded::encode_string(uuid.as_bytes())
}

// endregion: --- v4

// region:    --- v7

/// Generates a new UUID version 7 and encodes it using standard Base64.
pub fn new_v7_b64() -> String {
  let uuid = now_v7();
  Base64::encode_string(uuid.as_bytes())
}

/// Generates a new UUID version 7 and encodes it using URL-safe Base64.
pub fn new_v7_b64url() -> String {
  let uuid = now_v7();
  Base64Url::encode_string(uuid.as_bytes())
}

/// Generates a new UUID version 7 and encodes it using URL-safe Base64 without padding.
pub fn new_v7_b64url_nopad() -> String {
  let uuid = now_v7();
  Base64UrlUnpadded::encode_string(uuid.as_bytes())
}

// endregion: --- v7

// region:    --- From String

/// Decodes a standard Base64 encoded string into a UUID.
pub fn from_b64(s: &str) -> Result<Uuid, Error> {
  let decoded_bytes = Base64::decode_vec(s).map_err(Error::custom_from_err)?;
  support::from_vec_u8(decoded_bytes, "base64")
}

/// Decodes a URL-safe Base64 encoded string (with padding) into a UUID.
pub fn from_b64url(s: &str) -> Result<Uuid, Error> {
  let decoded_bytes = Base64Url::decode_vec(s).map_err(Error::custom_from_err)?;
  support::from_vec_u8(decoded_bytes, "base64url")
}

/// Decodes a URL-safe Base64 encoded string (without padding) into a UUID.
pub fn from_b64url_nopad(s: &str) -> Result<Uuid, Error> {
  let decoded_bytes = Base64UrlUnpadded::decode_vec(s).map_err(Error::custom_from_err)?;
  support::from_vec_u8(decoded_bytes, "base64url-nopad")
}

// endregion: --- From String

// region:    --- Tests

#[cfg(test)]
mod tests {
  type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

  use super::*;
  use uuid::{Uuid, Version};

  #[test]
  fn test_extra_base64_new_v4_b64_simple() -> Result<()> {
    // -- Setup & Fixtures
    // (no specific setup needed for this test)

    // -- Exec
    let b64_uuid = new_v4_b64();

    // -- Check
    assert_eq!(b64_uuid.len(), 24, "Standard Base64 of UUID should be 24 chars with padding");
    assert!(b64_uuid.ends_with("=="), "Standard Base64 should be padded with '==' for 16 bytes");
    assert!(
      !b64_uuid.contains('-') && !b64_uuid.contains('_'),
      "Standard Base64 should not contain URL-safe characters '-' or '_'"
    );

    // Decode and verify UUID
    let decoded_bytes = Base64::decode_vec(&b64_uuid)?;
    let uuid = Uuid::from_bytes(decoded_bytes.try_into().map_err(|_| "Failed to convert vec to array")?);
    assert_eq!(uuid.get_version(), Some(Version::Random));
    Ok(())
  }

  #[test]
  fn test_extra_base64_new_v4_b64url_simple() -> Result<()> {
    // -- Setup & Fixtures
    // (no specific setup needed for this test)

    // -- Exec
    let b64url_uuid = new_v4_b64url();

    // -- Check
    assert_eq!(b64url_uuid.len(), 24, "URL-safe Base64 of UUID should be 24 chars with padding");
    assert!(b64url_uuid.ends_with("=="), "URL-safe Base64 with padding should be padded with '==' for 16 bytes");
    assert!(!b64url_uuid.contains('+') && !b64url_uuid.contains('/'), "URL-safe Base64 should not contain '+' or '/'");

    // Decode and verify UUID
    let decoded_bytes = Base64Url::decode_vec(&b64url_uuid)?;
    let uuid = Uuid::from_bytes(decoded_bytes.try_into().map_err(|_| "Failed to convert vec to array")?);
    assert_eq!(uuid.get_version(), Some(Version::Random));
    Ok(())
  }

  #[test]
  fn test_extra_base64_new_v4_b64url_nopad_simple() -> Result<()> {
    // -- Setup & Fixtures
    // (no specific setup needed for this test)

    // -- Exec
    let b64url_nopad_uuid = new_v4_b64url_nopad();

    // -- Check
    assert_eq!(b64url_nopad_uuid.len(), 22, "URL-safe Base64 (no pad) of UUID should be 22 chars");
    assert!(!b64url_nopad_uuid.ends_with('='), "URL-safe Base64 (no pad) should not have padding");
    assert!(
      !b64url_nopad_uuid.contains('+') && !b64url_nopad_uuid.contains('/'),
      "URL-safe Base64 (no pad) should not contain '+' or '/'"
    );

    // Decode and verify UUID
    let decoded_bytes = Base64UrlUnpadded::decode_vec(&b64url_nopad_uuid)?;
    let uuid = Uuid::from_bytes(decoded_bytes.try_into().map_err(|_| "Failed to convert vec to array")?);
    assert_eq!(uuid.get_version(), Some(Version::Random));
    Ok(())
  }

  #[test]
  fn test_extra_base64_new_v7_b64_simple() -> Result<()> {
    // -- Setup & Fixtures
    // (no specific setup needed for this test)

    // -- Exec
    let b64_uuid = new_v7_b64();

    // -- Check
    assert_eq!(b64_uuid.len(), 24, "Standard Base64 of V7 UUID should be 24 chars with padding");
    assert!(b64_uuid.ends_with("=="), "Standard Base64 should be padded with '==' for 16 bytes");
    assert!(
      !b64_uuid.contains('-') && !b64_uuid.contains('_'),
      "Standard Base64 should not contain URL-safe characters '-' or '_'"
    );

    // Decode and verify UUID
    let decoded_bytes = Base64::decode_vec(&b64_uuid)?;
    let uuid = Uuid::from_bytes(decoded_bytes.try_into().map_err(|_| "Failed to convert vec to array")?);
    assert_eq!(uuid.get_version(), Some(Version::SortRand));
    Ok(())
  }

  #[test]
  fn test_extra_base64_new_v7_b64url_simple() -> Result<()> {
    // -- Setup & Fixtures
    // (no specific setup needed for this test)

    // -- Exec
    let b64url_uuid = new_v7_b64url();

    // -- Check
    assert_eq!(b64url_uuid.len(), 24, "URL-safe Base64 of V7 UUID should be 24 chars with padding");
    assert!(b64url_uuid.ends_with("=="), "URL-safe Base64 with padding should be padded with '==' for 16 bytes");
    assert!(!b64url_uuid.contains('+') && !b64url_uuid.contains('/'), "URL-safe Base64 should not contain '+' or '/'");

    // Decode and verify UUID
    let decoded_bytes = Base64Url::decode_vec(&b64url_uuid)?;
    let uuid = Uuid::from_bytes(decoded_bytes.try_into().map_err(|_| "Failed to convert vec to array")?);
    assert_eq!(uuid.get_version(), Some(Version::SortRand));
    Ok(())
  }

  #[test]
  fn test_extra_base64_new_v7_b64url_nopad_simple() -> Result<()> {
    // -- Setup & Fixtures
    // (no specific setup needed for this test)

    // -- Exec
    let b64url_nopad_uuid = new_v7_b64url_nopad();

    // -- Check
    assert_eq!(b64url_nopad_uuid.len(), 22, "URL-safe Base64 (no pad) of V7 UUID should be 22 chars");
    assert!(!b64url_nopad_uuid.ends_with('='), "URL-safe Base64 (no pad) should not have padding");
    assert!(
      !b64url_nopad_uuid.contains('+') && !b64url_nopad_uuid.contains('/'),
      "URL-safe Base64 (no pad) should not contain '+' or '/'"
    );

    // Decode and verify UUID
    let decoded_bytes = Base64UrlUnpadded::decode_vec(&b64url_nopad_uuid)?;
    let uuid = Uuid::from_bytes(decoded_bytes.try_into().map_err(|_| "Failed to convert vec to array")?);
    assert_eq!(uuid.get_version(), Some(Version::SortRand));
    Ok(())
  }

  // region:    --- Tests for from_... functions

  #[test]
  fn test_extra_base64_from_b64_ok() -> Result<()> {
    // -- Setup & Fixtures
    let original_uuid = Uuid::now_v7();
    let b64_string = Base64::encode_string(original_uuid.as_bytes());

    // -- Exec
    let decoded_uuid_res = from_b64(&b64_string);

    // -- Check
    assert!(decoded_uuid_res.is_ok(), "Decoding should succeed");
    assert_eq!(decoded_uuid_res.unwrap(), original_uuid, "Decoded UUID should match original");
    Ok(())
  }

  #[test]
  fn test_extra_base64_from_b64_err_invalid_char() -> Result<()> {
    // -- Setup & Fixtures
    let invalid_b64_string = "ThisIsNotValidBase64!=";

    // -- Exec
    let decoded_uuid_res = from_b64(invalid_b64_string);

    // -- Check
    assert!(decoded_uuid_res.is_err(), "Decoding should fail for invalid characters");
    let err_msg = decoded_uuid_res.err().unwrap().to_string();
    // base64ct 库的错误消息可能包含这些字符串之一
    assert!(
      err_msg.contains("invalid") || err_msg.contains("Invalid") || err_msg.contains("symbol"),
      "Error message should indicate invalid character, but got: {}",
      err_msg
    );
    Ok(())
  }

  #[test]
  fn test_extra_base64_from_b64_err_wrong_len() -> Result<()> {
    // -- Setup & Fixtures
    let short_b64_string = Base64::encode_string("short".as_bytes()); // Decodes to 5 bytes

    // -- Exec
    let decoded_uuid_res = from_b64(&short_b64_string);

    // -- Check
    assert!(decoded_uuid_res.is_err(), "Decoding should fail for wrong length");
    let err_msg = decoded_uuid_res.err().unwrap().to_string();
    println!("->> {err_msg}");
    // 检查错误消息包含相关的长度错误信息
    assert!(
      err_msg.contains("Fail to decode 16U8") && err_msg.contains("base64") && err_msg.contains("actual length: 5"),
      "Error message should indicate wrong length, but got: {}",
      err_msg
    );
    Ok(())
  }

  #[test]
  fn test_extra_base64_from_b64url_ok() -> Result<()> {
    // -- Setup & Fixtures
    let original_uuid = Uuid::now_v7();
    let b64url_string = Base64Url::encode_string(original_uuid.as_bytes());

    // -- Exec
    let decoded_uuid_res = from_b64url(&b64url_string);

    // -- Check
    assert!(decoded_uuid_res.is_ok(), "Decoding should succeed");
    assert_eq!(decoded_uuid_res.unwrap(), original_uuid, "Decoded UUID should match original");
    Ok(())
  }

  #[test]
  fn test_extra_base64_from_b64url_err_invalid_char() -> Result<()> {
    // -- Setup & Fixtures
    let invalid_b64url_string = "ThisIsNotValidBase64Url+"; // '+' is not valid for URL_SAFE if not part of encoding

    // -- Exec
    let decoded_uuid_res = from_b64url(invalid_b64url_string);

    // -- Check
    assert!(decoded_uuid_res.is_err(), "Decoding should fail for invalid characters");
    let err_msg = decoded_uuid_res.err().unwrap().to_string();
    // 验证错误确实发生了，不强制要求特定的错误消息
    assert!(!err_msg.is_empty(), "Error message should not be empty");
    Ok(())
  }

  #[test]
  fn test_extra_base64_from_b64url_err_wrong_len() -> Result<()> {
    // -- Setup & Fixtures
    let short_b64url_string = Base64Url::encode_string("short".as_bytes()); // Decodes to 5 bytes

    // -- Exec
    let decoded_uuid_res = from_b64url(&short_b64url_string);

    // -- Check
    assert!(decoded_uuid_res.is_err(), "Decoding should fail for wrong length");
    let err_msg = decoded_uuid_res.err().unwrap().to_string();
    // 检查包含长度错误信息
    assert!(
      err_msg.contains("Fail to decode 16U8") && err_msg.contains("base64url") && err_msg.contains("actual length: 5"),
      "Error message should indicate FailToDecode16U8 for base64url, but got: {}",
      err_msg
    );
    Ok(())
  }

  #[test]
  fn test_extra_base64_from_b64url_nopad_ok() -> Result<()> {
    // -- Setup & Fixtures
    let original_uuid = Uuid::now_v7();
    let b64url_nopad_string = Base64UrlUnpadded::encode_string(original_uuid.as_bytes());

    // -- Exec
    let decoded_uuid_res = from_b64url_nopad(&b64url_nopad_string);

    // -- Check
    assert!(decoded_uuid_res.is_ok(), "Decoding should succeed");
    assert_eq!(decoded_uuid_res.unwrap(), original_uuid, "Decoded UUID should match original");
    Ok(())
  }

  #[test]
  fn test_extra_base64_from_b64url_nopad_err_invalid_char() -> Result<()> {
    // -- Setup & Fixtures
    let invalid_b64url_nopad_string = "ThisIsNotValidBase64UrlNoPad="; // '=' is not valid for NO_PAD

    // -- Exec
    let decoded_uuid_res = from_b64url_nopad(invalid_b64url_nopad_string);

    // -- Check
    assert!(decoded_uuid_res.is_err(), "Decoding should fail for invalid characters");
    let err_msg = decoded_uuid_res.err().unwrap().to_string();
    // 验证错误确实发生了，不强制要求特定的错误消息
    assert!(!err_msg.is_empty(), "Error message should not be empty");
    Ok(())
  }

  #[test]
  fn test_extra_base64_from_b64url_nopad_err_wrong_len() -> Result<()> {
    // -- Setup & Fixtures
    let short_b64url_nopad_string = Base64UrlUnpadded::encode_string("short".as_bytes());

    // -- Exec
    let decoded_uuid_res = from_b64url_nopad(&short_b64url_nopad_string);

    // -- Check
    assert!(decoded_uuid_res.is_err(), "Decoding should fail for wrong length");
    let err_msg = decoded_uuid_res.err().unwrap().to_string();
    // 检查包含长度错误信息
    assert!(
      err_msg.contains("Fail to decode 16U8")
        && err_msg.contains("base64url-nopad")
        && err_msg.contains("actual length: 5"),
      "Error message should indicate FailToDecode16U8 for base64url-nopad, but got: {}",
      err_msg
    );
    Ok(())
  }

  // endregion: --- Tests for from_... functions
}

// endregion: --- Tests
