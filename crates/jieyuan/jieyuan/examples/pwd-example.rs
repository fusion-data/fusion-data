use fusion_core::security::pwd::generate_pwd;

#[tokio::main]
async fn main() {
  let pwd = "2024.Fusiondata";

  let encrypted_pwd = generate_pwd(pwd).await.unwrap();
  println!("encrypted_pwd: {}", encrypted_pwd);
}
