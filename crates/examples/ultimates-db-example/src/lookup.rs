use std::net::ToSocketAddrs;

fn main() -> anyhow::Result<()> {
  let host = "node1:35432";
  let sock_iter = host.to_socket_addrs()?;
  for sock_addr in sock_iter {
    println!("socket address is {}", sock_addr);
  }

  Ok(())
}
