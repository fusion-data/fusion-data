use std::path::PathBuf;

fn main() {
  let path_str = r"D:\Documents\abc.pdf";
  println!("path_str: {:?}", path_str);
  let path = PathBuf::from(path_str);
  println!("path: {:?}", path);
  println!("path: {:?}", path.parent());
  println!("path: {:?}", path.file_name());
  println!("path: {:?}", path.extension());
  println!("path: {:?}", path.display());
  println!("path: {:?}", path.to_str());
}
