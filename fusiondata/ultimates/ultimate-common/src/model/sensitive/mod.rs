mod sensitive_string;
mod uri_string;

pub use sensitive_string::SensitiveString;
pub use uri_string::UriString;

pub trait ToSensitive {
  fn to_sensitive(&self) -> String;
}

pub trait AsUnderlying {
  fn as_underlying(&self) -> &str;
}
