use modelsql::DbConfig;

fn main() {
  let text = r#"{
    "enable": true,
    "max_lifetime": "30s",
    "url": "postgres://user:pass@localhost:5432/db"
  }"#;
  let conf: DbConfig = serde_json::from_str(text).unwrap();
  println!("{}", serde_json::to_string_pretty(&conf).unwrap());
}
