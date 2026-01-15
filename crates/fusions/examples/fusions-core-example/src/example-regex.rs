use regex::Regex;

fn main() {
  let re = Regex::new(r"Hello (?<name>\w+)!").unwrap();
  let Some(caps) = re.captures("Hello Murphy!") else {
    println!("no match!");
    return;
  };
  println!("The name is: {}", &caps["name"]);

  let re = Regex::new(r"#(?<version>\d+)#").unwrap();
  let pwd = "#11#789snd923rlasfdj-=_982347asffdl23";
  let Some(caps) = re.captures(pwd) else {
    println!("no version!");
    return;
  };
  println!("The version of pwd is {:?}", caps);
  let version = caps.name("version").unwrap();
  println!(
    "start:{}, end:{}, len:{}, range:{:?}, str:{}",
    version.start(),
    version.end(),
    version.len(),
    version.range(),
    version.as_str()
  );

  let hash = &pwd[version.end() + 1..];
  println!("hash is: {hash}");
}
