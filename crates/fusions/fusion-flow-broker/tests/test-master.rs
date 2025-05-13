use std::{process::exit, time::Duration};

use fusion_flow_broker::loop_master;
use ultimate_core::{application::Application, timer::TimerPlugin};
use ultimate_db::DbPlugin;

#[tokio::test]
async fn test_master_init() {
  Application::builder().add_plugin(TimerPlugin).add_plugin(DbPlugin).run().await.unwrap();
  let app = Application::global();

  // let (db_tx, db_rx) = mpsc::channel(1024);
  tokio::spawn(loop_master(app.clone()));

  tokio::time::sleep(Duration::from_secs(2)).await;
  println!("Sleeped 10 seconds, exiting...");
  exit(0);
}
