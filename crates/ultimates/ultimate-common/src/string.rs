use base64ct::{Base64UrlUnpadded, Encoding};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{de::Visitor, Deserializer, Serializer};

use crate::{Error, Result};

pub fn repeat_str(s: &str, n: usize) -> String {
  let mut v = String::with_capacity(s.len() * n);
  for _ in 0..n {
    v.push_str(s);
  }
  v
}

pub fn repeat_char(c: char, n: usize) -> String {
  let mut v = String::with_capacity(n);
  for _ in 0..n {
    v.push(c);
  }
  v
}

pub fn random_string(n: usize) -> String {
  thread_rng().sample_iter(&Alphanumeric).take(n).map(char::from).collect()
}

pub fn b64u_encode(content: impl AsRef<[u8]>) -> String {
  Base64UrlUnpadded::encode_string(content.as_ref())
}

pub fn b64u_decode(b64u: &str) -> Result<Vec<u8>> {
  Base64UrlUnpadded::decode_vec(b64u).map_err(|_| Error::FailToB64uDecode(format!("Input string: {b64u}")))
}

pub fn b64u_decode_to_string(b64u: &str) -> Result<String> {
  b64u_decode(b64u)
    .ok()
    .and_then(|r| String::from_utf8(r).ok())
    .ok_or(Error::FailToB64uDecode(format!("Input string: {b64u}")))
}

pub fn deser_str_to_vecu8<'de, D>(d: D) -> core::result::Result<Vec<u8>, D::Error>
where
  D: Deserializer<'de>,
{
  struct StrToVecU8;
  impl<'d> Visitor<'d> for StrToVecU8 {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
      formatter.write_str("expect 'str'.")
    }

    fn visit_str<E>(self, v: &str) -> core::result::Result<Self::Value, E>
    where
      E: serde::de::Error,
    {
      Ok(v.as_bytes().into())
    }
  }

  d.deserialize_str(StrToVecU8)
}

pub fn ser_vecu8_to_str<S>(v: &[u8], s: S) -> core::result::Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let string = std::str::from_utf8(v).map_err(serde::ser::Error::custom)?;
  s.serialize_str(string)
}

/// 对需要保密的数据进行脱敏
pub fn ser_str_secret<S>(v: &str, s: S) -> core::result::Result<S::Ok, S::Error>
where
  S: Serializer,
{
  s.serialize_str(v)
}

pub fn ser_opt_str_secret<S>(v: Option<String>, s: S) -> core::result::Result<S::Ok, S::Error>
where
  S: Serializer,
{
  match v {
    Some(v) => s.serialize_str(&v),
    None => s.serialize_none(),
  }
}
