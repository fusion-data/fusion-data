use tokio::process::Command;
use hetus::core::DataError;

#[tokio::main]
async fn main() -> Result<(), DataError> {
  let mut cmd = Command::new("bash");
  cmd.args(["-c", "echo '你好，河图'"]);

  println!("Cmd: {:?}", cmd);

  let ret = cmd.output().await?;

  println!("{:?}", ret);

  Ok(())
}
