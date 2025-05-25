fn add(a: u128, b: u128) -> u128 {
  // a + b
  a.wrapping_add(b)
}

fn main() {
  let i: u128 = 200;
  let j = u128::MAX;
  let r = add(i, j);

  println!("r = {}", r);

  println!("{}", j);
  println!("{}", u64::MAX / (1000 * 60 * 60 * 24 * 365));

  let mut v1 = vec![1, 2, 3, 4];
  let mut v2 = Vec::with_capacity(2);
  println!("v1: {:?}, capacity: {}", v1, v1.capacity());
  println!("v2: {:?}, capacity: {}", v2, v2.capacity());
  core::mem::swap(&mut v1, &mut v2);
  println!("v1: {:?}, capacity: {}", v1, v1.capacity());
  println!("v2: {:?}, capacity: {}", v2, v2.capacity());
}
