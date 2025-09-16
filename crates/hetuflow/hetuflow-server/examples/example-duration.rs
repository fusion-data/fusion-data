use std::time::Duration;

fn main() {
  let d = duration_str::parse("3m").unwrap();
  println!("{:?}", d);
  assert_eq!(d, Duration::from_secs(3 * 60));
}
