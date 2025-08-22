use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
  Get,
  Post,
  Put,
  Delete,
  Patch,
  Head,
  Options,
}

impl AsRef<str> for HttpMethod {
  fn as_ref(&self) -> &str {
    match self {
      HttpMethod::Get => "GET",
      HttpMethod::Post => "POST",
      HttpMethod::Put => "PUT",
      HttpMethod::Delete => "DELETE",
      HttpMethod::Patch => "PATCH",
      HttpMethod::Head => "HEAD",
      HttpMethod::Options => "OPTIONS",
    }
  }
}

impl<'de> Deserialize<'de> for HttpMethod {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    match s.to_uppercase().as_str() {
      "GET" => Ok(HttpMethod::Get),
      "POST" => Ok(HttpMethod::Post),
      "PUT" => Ok(HttpMethod::Put),
      "DELETE" => Ok(HttpMethod::Delete),
      "PATCH" => Ok(HttpMethod::Patch),
      "HEAD" => Ok(HttpMethod::Head),
      "OPTIONS" => Ok(HttpMethod::Options),
      _ => Err(serde::de::Error::custom(format!("不支持的 HTTP 方法: {}", s))),
    }
  }
}

impl HttpMethod {
  pub fn create_request_builder(&self, client: &Client, url: &str) -> RequestBuilder {
    match self {
      HttpMethod::Get => client.get(url),
      HttpMethod::Post => client.post(url),
      HttpMethod::Put => client.put(url),
      HttpMethod::Delete => client.delete(url),
      HttpMethod::Patch => client.patch(url),
      HttpMethod::Head => client.head(url),
      HttpMethod::Options => client.request(Method::OPTIONS, url),
    }
  }

  pub const ALL: &[HttpMethod] = &[
    HttpMethod::Get,
    HttpMethod::Post,
    HttpMethod::Put,
    HttpMethod::Delete,
    HttpMethod::Patch,
    HttpMethod::Head,
    HttpMethod::Options,
  ];
}
