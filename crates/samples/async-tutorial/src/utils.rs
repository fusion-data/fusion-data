use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

pub async fn handle_connection<Reader, Writer>(reader: Reader, mut writer: Writer) -> std::io::Result<()>
where
  Reader: AsyncRead + Unpin,
  Writer: AsyncWrite + Unpin,
{
  let mut line = String::new();
  let mut reader = BufReader::new(reader);

  loop {
    if let Ok(bytes_read) = reader.read_line(&mut line).await {
      if bytes_read == 0 {
        break Ok(());
      }
      writer.write_all("Thanks for your message.\r\n".as_bytes()).await.unwrap();
    }
    line.clear();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // 使用 tokio_test::io::Builder::new() 创建一个模拟的 Reader 和 Writer
  #[tokio::test]
  async fn test_handle_connection() {
    let reader = tokio_test::io::Builder::new().read(b"Hi there\r\n").read(b"How are you doing?\r\n").build();
    let writer = tokio_test::io::Builder::new()
      .write(b"Thanks for your message.\r\n")
      .write(b"Thanks for your message.\r\n")
      .build();
    let _ = handle_connection(reader, writer).await;
  }
}
