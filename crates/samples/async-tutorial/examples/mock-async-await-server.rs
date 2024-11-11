use async_tutorial::utils::handle_connection;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
  loop {
    let Ok((mut socket, _)) = listener.accept().await else {
      eprintln!("Failed to accept client");
      continue;
    };

    tokio::spawn(async move {
      let (reader, writer) = socket.split();
      // Run some client connection handler, for example:
      handle_connection(reader, writer).await.expect("Failed to handle connection");
    });
  }
}
