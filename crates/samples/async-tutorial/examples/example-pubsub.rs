use std::time::Duration;

use mini_redis::client;
use tokio_stream::StreamExt;

async fn publish() -> mini_redis::Result<()> {
  let mut client = client::connect("127.0.0.1:6379").await?;

  // Publish some data
  client.publish("numbers", "1".into()).await?;
  client.publish("numbers", "two".into()).await?;
  client.publish("numbers", "3".into()).await?;
  client.publish("numbers", "four".into()).await?;
  client.publish("numbers", "five".into()).await?;
  client.publish("numbers", "6".into()).await?;
  Ok(())
}

async fn subscribe() -> mini_redis::Result<()> {
  let client = client::connect("127.0.0.1:6379").await?;
  let subscriber = client.subscribe(vec!["numbers".to_string()]).await?;
  let messages = subscriber
    .into_stream()
    // .filter(|msg| matches!(msg, Ok(msg) if msg.content.len() == 1))
    // .map(|msg| msg.unwrap().content)
    .filter_map(|msg| match msg {
      Ok(msg) if msg.content.len() == 1 => Some(msg.content),
      _ => None,
    })
    .take(3);

  tokio::pin!(messages);

  while let Some(msg) = messages.next().await {
    println!("got = {:?}", msg);
  }

  Ok(())
}

#[tokio::main]
async fn main() -> mini_redis::Result<()> {
  tokio::spawn(async {
    tokio::time::sleep(Duration::from_millis(100)).await;
    publish().await
  });

  subscribe().await?;

  println!("DONE");

  Ok(())
}
