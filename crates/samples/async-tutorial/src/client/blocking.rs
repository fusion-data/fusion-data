use std::time::Duration;

use bytes::Bytes;
use mini_redis::{
  client::{connect, Client, Message, Subscriber},
  Result,
};
use tokio::{net::ToSocketAddrs, runtime::Runtime};

/// Established connection with a Redis Server.
pub struct BlockingClient {
  /// The asynchronous `Client`.
  inner: Client,

  /// A `current_thread` runtime for executing operations on the asynchronous client
  /// in a blocking manner.
  rt: Runtime,
}

impl BlockingClient {
  pub fn connect<T: ToSocketAddrs>(addr: T) -> mini_redis::Result<Self> {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

    // Call the asynchronous connect method using the runtime.
    let inner = rt.block_on(connect(addr))?;

    Ok(Self { inner, rt })
  }

  pub fn subscribe(self, channels: Vec<String>) -> Result<BlockingSubscriber> {
    let subscriber = self.rt.block_on(self.inner.subscribe(channels))?;
    Ok(BlockingSubscriber { inner: subscriber, rt: self.rt })
  }

  pub fn get(&mut self, key: &str) -> Result<Option<Bytes>> {
    self.rt.block_on(self.inner.get(key))
  }

  pub fn set(&mut self, key: &str, value: Bytes) -> Result<()> {
    self.rt.block_on(self.inner.set(key, value))
  }

  pub fn set_expires(&mut self, key: &str, value: Bytes, expiration: Duration) -> Result<()> {
    self.rt.block_on(self.inner.set_expires(key, value, expiration))
  }

  pub fn publish(&mut self, channel: &str, message: Bytes) -> Result<u64> {
    self.rt.block_on(self.inner.publish(channel, message))
  }
}

/// A client that has entered pub/sub mode.
///
/// Once clients subscribe to a channel, they may only perform pub/sub related commands.
/// The `BlockingClient` type is transitioned to a `BlockingSubscriber` type in order to
/// prevent non-pub/sub methods from being called.
pub struct BlockingSubscriber {
  /// The asynchronous `Subscriber`.
  inner: Subscriber,

  /// A `current_thread` runtime for executing operations on the asynchronous client
  /// in a blocking manner.
  rt: Runtime,
}

impl BlockingSubscriber {
  pub fn get_subscribed(&self) -> &[String] {
    self.inner.get_subscribed()
  }

  pub fn next_message(&mut self) -> Result<Option<Message>> {
    self.rt.block_on(self.inner.next_message())
  }

  pub fn subscribe(&mut self, channels: &[String]) -> Result<()> {
    self.rt.block_on(self.inner.subscribe(channels))
  }

  pub fn unsubscribe(&mut self, channels: &[String]) -> Result<()> {
    self.rt.block_on(self.inner.unsubscribe(channels))
  }
}
