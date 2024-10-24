use std::{
  collections::HashMap,
  sync::{Arc, Mutex, MutexGuard},
};

use bytes::Bytes;
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
  // Bind the listener to the address
  let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

  println!("Listening on {}", listener.local_addr().unwrap());

  let db: Db = Arc::new(Mutex::new(HashMap::new()));

  loop {
    // The second item contains the IP and port of the new connection.
    let (socket, _) = listener.accept().await.unwrap();
    println!("Accepted a new connection: {:?}", socket);

    let db = db.clone();

    // A new task is spawed for each inbound socket. The socket is
    // moved to the new task and processed there.
    tokio::spawn(process(socket, db));
  }
}

async fn process(socket: TcpStream, db: Db) {
  use mini_redis::Command;

  // The `Connection` lets us read/write redis **frames** instead of
  // byte streams. The `Connection` type is defined by mini-redis
  let mut connection = Connection::new(socket);

  // Use `read_frame` to receive a command from the connection.
  while let Some(frame) = connection.read_frame().await.unwrap() {
    println!("Got frame: {:?}", frame);

    let response = match Command::from_frame(frame).unwrap() {
      Command::Set(cmd) => {
        let mut db = db.lock().unwrap();
        // The value is stored as `Vec<u8>`
        db.insert(cmd.key().to_string(), cmd.value().clone());
        Frame::Simple("OK".to_string())
      }
      Command::Get(cmd) => {
        let db = db.lock().unwrap();
        if let Some(value) = db.get(cmd.key()) {
          // `Frame::Bluk` expects data to be of type `Bytes`. This type will be
          // covered later in the tutorial. For now, `&Vec<u8>` is converted to
          // `Bytes` using `into()`.
          Frame::Bulk(value.clone())
        } else {
          Frame::Null
        }
      }
      cmd => panic!("unimplemented {:?}", cmd),
    };

    // Write the reponse to the client
    connection.write_frame(&response).await.unwrap();
  }
}

async fn increment_and_do_stuff(mutex: &Mutex<i32>) {
  {
    let mut lock: MutexGuard<i32> = mutex.lock().unwrap();
    *lock += 1;
  }

  do_something_async().await;
} // lock goes out of scope here

async fn do_something_async() {
  println!("Doing something async");
}
