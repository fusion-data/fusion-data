use std::env;

fn main() {
  for e in env::vars() {
    println!("{:?}", e);
  }
}
