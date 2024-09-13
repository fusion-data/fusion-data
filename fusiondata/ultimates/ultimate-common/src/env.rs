use std::env;
use std::str::FromStr;

use crate::digest::b64u_decode;
use crate::Error;

pub fn get_env(name: &'static str) -> Result<String, Error> {
  env::var(name).map_err(|_| Error::MissingEnv(name))
}

pub fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T, Error> {
  let val = get_env(name)?;
  val.parse::<T>().map_err(|_| Error::WrongFormat(name))
}

pub fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>, Error> {
  b64u_decode(&get_env(name)?).map_err(|_| Error::WrongFormat(name))
}
