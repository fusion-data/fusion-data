use std::cell::Cell;

fn f(a: &Cell<i32>, b: &Cell<i32>) {
  let before = a.get();
  b.set(b.get() + 1);
  let after = a.get();
  println!("a: {:?}", a);
  println!("b: {:?}", b);
  if before != after {
    println!("a was changed")
  }
}

// 内部可变性示例
fn ff(v: &Cell<Vec<i32>>) {
  let mut v2 = v.take();
  v2.push(1);
  v.set(v2);
}

fn main() {
  let x = 8;
  let a = Cell::new(x);
  let b = Cell::new(x);

  f(&a, &b);

  let vv = Cell::new(vec![3, 2]);
  ff(&vv);
  let v = vv.take();
  println!("vv: {:?}", v);
}
