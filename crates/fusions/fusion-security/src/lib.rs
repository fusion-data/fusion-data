pub mod jwt;

#[cfg(feature = "with-oauth")]
pub mod oauth;

#[cfg(feature = "with-oauth")]
pub use oauth2;
#[cfg(feature = "with-openid")]
pub use openidconnect;
