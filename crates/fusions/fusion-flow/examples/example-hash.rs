use std::{
  fmt::Debug,
  hash::{DefaultHasher, Hash, Hasher},
  net::{IpAddr, SocketAddr},
  str::FromStr,
};

#[derive(Debug, Hash)]
struct SN {
  tenant_id: i32,
  namespace: String,
}

fn main() {
  let sn = SN { tenant_id: 1, namespace: "default".to_string() };
  hash_to_node(&sn);
  let sn2 = SN { tenant_id: 2, namespace: "default".to_string() };
  hash_to_node(&sn2);
  let sn3 = SN { tenant_id: 1, namespace: "default2".to_string() };
  hash_to_node(&sn3);
  let sn4 = SN { tenant_id: 2, namespace: "default2".to_string() };
  hash_to_node(&sn4);

  let addr = SocketAddr::new(IpAddr::from_str("10.0.10.121").unwrap(), 4000);
  let (hash, _) = hash_to_node(&addr);
  let hi: i64 = hash as i64;
  println!("hash: {}, hi: {}", hash, hi);
  let u64_max = u64::MAX;
  let u64_max_i64: i64 = u64_max as i64 - 234234234;
  println!("{} - {}", u64_max, u64_max_i64);
}

/// 返回 (hash值, mod value)
fn hash_to_node<H: Hash + Debug>(sn: &H) -> (u64, u64) {
  let mut hasher = DefaultHasher::new();
  sn.hash(&mut hasher);
  let hash = hasher.finish();
  let m = hash % 3;
  println!("{:?} mod: {} hash: {}", sn, m, hash);
  (hash, m)
}
