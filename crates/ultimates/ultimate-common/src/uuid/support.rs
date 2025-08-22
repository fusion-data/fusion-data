use uuid::Uuid;

use super::Error;

pub fn from_vec_u8(decoded_bytes: Vec<u8>, error_context: &'static str) -> Result<Uuid, Error> {
  let bytes_array: [u8; 16] = decoded_bytes
    .try_into()
    .map_err(|ex: Vec<u8>| Error::FailToDecode16U8 { context: error_context, actual_length: ex.len() })?;

  Ok(Uuid::from_bytes(bytes_array))
}
