use tokio::{io, net::TcpListener};

#[tokio::main]
async fn main() -> io::Result<()> {
  let listener = TcpListener::bind("127.0.1:6142").await?;

  loop {
    let (mut socket, _) = listener.accept().await?;

    tokio::spawn(async move {
      // Copy data here
      let (mut rd, mut wr) = socket.split();

      if let Err(e) = io::copy(&mut rd, &mut wr).await {
        eprintln!("Failed to copy: {}", e);
      }
    });
  }
}
