mod time;
mod uri_string;

pub use time::now_offset;
pub use uri_string::UriString;

pub trait ToSensitive {
  fn to_sensitive(&self) -> String;
}

pub trait AsUnderlying {
  fn as_underlying(&self) -> &str;
}
