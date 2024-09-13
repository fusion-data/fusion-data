use base64ct::{Base64UrlUnpadded, Encoding};
pub use hmac::digest::InvalidLength;
use hmac::{
  digest::{
    generic_array::GenericArray,
    typenum::{UInt, UTerm, B0, B1},
    CtOutput,
  },
  Hmac, Mac,
};
use sha2::{Digest, Sha256};

use crate::Error;

type HmacSha256 = Hmac<Sha256>;

pub fn hmac_sha256(secret: &[u8], s: &[u8]) -> Result<CtOutput<HmacSha256>, InvalidLength> {
  let mut mac = HmacSha256::new_from_slice(secret)?;
  mac.update(s);
  let result = mac.finalize();
  Ok(result)
}

#[inline]
pub fn hmac_sha256_string(secret: &[u8], s: &[u8]) -> Result<String, InvalidLength> {
  let bytes = hmac_sha256(secret, s)?.into_bytes();
  Ok(base16ct::lower::encode_string(&bytes))
}

type ShaArray = GenericArray<u8, UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>>;

pub fn sha256(s: &[u8]) -> ShaArray {
  let mut hasher: Sha256 = Sha256::new();
  hasher.update(s);
  hasher.finalize()
}

#[inline]
pub fn sha256_string(s: &[u8]) -> String {
  let result = sha256(s);
  base16ct::lower::encode_string(&result)
}

pub fn b64u_encode(content: impl AsRef<[u8]>) -> String {
  Base64UrlUnpadded::encode_string(content.as_ref())
}

pub fn b64u_decode(b64u: &str) -> Result<Vec<u8>, Error> {
  Base64UrlUnpadded::decode_vec(b64u).map_err(|_| Error::FailToB64uDecode(b64u.to_string()))
}

pub fn b64u_decode_to_string(b64u: &str) -> Result<String, Error> {
  b64u_decode(b64u).ok().and_then(|r| String::from_utf8(r).ok()).ok_or(Error::FailToB64uDecode(b64u.to_string()))
}
