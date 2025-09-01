use std::env;
use std::str::FromStr;

use crate::Error;
use crate::digest::b64u_decode;

pub fn get_env(name: &str) -> Result<String, Error> {
  env::var(name).map_err(|_| Error::MissingEnv(name.to_string()))
}

pub fn get_env_parse<T: FromStr>(name: &str) -> Result<T, Error> {
  let val = get_env(name)?;
  val.parse::<T>().map_err(|_| Error::WrongFormat(name.to_string()))
}

pub fn get_envs() -> Vec<(String, String)> {
  std::env::vars().collect()
}

pub fn get_env_b64u_as_u8s(name: &str) -> Result<Vec<u8>, Error> {
  b64u_decode(&get_env(name)?).map_err(|_| Error::WrongFormat(name.to_string()))
}

pub fn set_env(name: &str, value: &str) -> Result<(), Error> {
  std::panic::catch_unwind(|| unsafe {
    env::set_var(name, value);
  })
  .map_err(|e| {
    let leaked_value: &'static str = Box::leak(value.to_string().into_boxed_str());
    Error::FailedToSetEnv(name.to_string(), leaked_value.to_string(), panic_message(e))
  })
}

pub fn remove_env(name: &'static str) -> Result<(), Error> {
  std::panic::catch_unwind(|| unsafe {
    env::remove_var(name);
  })
  .map_err(|e| Error::FailedToRemoveEnv(name.to_string(), panic_message(e)))
}

fn panic_message(e: Box<dyn std::any::Any + Send>) -> String {
  let any = &*e as &dyn std::any::Any;
  if let Some(s) = any.downcast_ref::<String>() {
    return s.clone();
  }
  if let Some(s) = any.downcast_ref::<&'static str>() {
    return (*s).to_string();
  }
  "panic with non-string payload".to_string()
}
