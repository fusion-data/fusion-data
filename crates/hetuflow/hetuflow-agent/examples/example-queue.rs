use mea::mpsc;

#[tokio::main]
async fn main() {
  demo_mea_mpsc().await;
}

async fn demo_mea_mpsc() {
  let (tx, mut rx) = mpsc::unbounded();
  tx.send(1).unwrap();
  let v = rx.recv().await.unwrap();
  println!("v: {:?}", v);
}
