use super::{ArrayBool, ArrayDouble, ArrayInt32, ArrayInt64, ArrayString};

impl<I> From<I> for ArrayBool
where
  I: IntoIterator<Item = bool>,
{
  fn from(value: I) -> Self {
    Self { value: value.into_iter().collect() }
  }
}

impl<I> From<I> for ArrayString
where
  I: IntoIterator<Item = String>,
{
  fn from(value: I) -> Self {
    Self { value: value.into_iter().collect() }
  }
}

impl<I> From<I> for ArrayInt32
where
  I: IntoIterator<Item = i32>,
{
  fn from(value: I) -> Self {
    Self { value: value.into_iter().collect() }
  }
}

impl<I> From<I> for ArrayInt64
where
  I: IntoIterator<Item = i64>,
{
  fn from(value: I) -> Self {
    Self { value: value.into_iter().collect() }
  }
}

impl<I> From<I> for ArrayDouble
where
  I: IntoIterator<Item = f64>,
{
  fn from(value: I) -> Self {
    Self { value: value.into_iter().collect() }
  }
}
