use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
  Get { key: String, resp: Responder<Option<Bytes>> },
  Set { key: String, val: Bytes, resp: Responder<()> },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
  // Create a new channel with a capacity of at most 32.
  let (tx, mut rx) = mpsc::channel(32);
  let tx2 = tx.clone();

  // Spawn two tasks, one gets a key, the other sets a key
  let t1 = tokio::spawn(async move {
    let (resp, resp_rx) = oneshot::channel();
    tx.send(Command::Set { key: "foo".into(), val: "bar".into(), resp }).await.unwrap();

    // Await the response
    let res = resp_rx.await;
    println!("GOT = {:?}", res);
  });
  let t2 = tokio::spawn(async move {
    let (resp, resp_rx) = oneshot::channel();
    tx2.send(Command::Get { key: "foo".into(), resp }).await.unwrap();

    // Await the response
    let res = resp_rx.await;
    println!("GOT = {:?}", res);
  });

  let manager = tokio::spawn(async move {
    // Establish a connection to the server
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    while let Some(cmd) = rx.recv().await {
      println!("GOT =: {:?}", cmd);

      match cmd {
        Command::Get { key, resp } => {
          let res = client.get(&key).await;
          // Ignore errors
          let _ = resp.send(res);
        }
        Command::Set { key, val, resp } => {
          let res = client.set(&key, val).await;
          // Ignore errors
          let _ = resp.send(res);
        }
      }
    }
  });

  t1.await.unwrap();
  t2.await.unwrap();
  manager.await.unwrap();
}
