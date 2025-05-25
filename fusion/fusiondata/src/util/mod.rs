use core::{
  fmt::{Display, Formatter},
  net::SocketAddr,
};

#[derive(Debug, Clone, Hash)]
pub struct HostAddr {
  host: String,
  port: u16,
}

impl HostAddr {
  pub fn host(&self) -> &str {
    &self.host
  }

  pub fn port(&self) -> u16 {
    self.port
  }
}

impl Display for HostAddr {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}:{}", self.host, self.port)
  }
}

impl From<SocketAddr> for HostAddr {
  fn from(value: SocketAddr) -> Self {
    HostAddr { host: value.ip().to_string(), port: value.port() }
  }
}
