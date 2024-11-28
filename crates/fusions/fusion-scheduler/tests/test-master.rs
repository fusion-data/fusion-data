use fusion_scheduler::broker::loop_master;
use ultimate::application::Application;
use ultimate_db::DbPlugin;

#[tokio::test]
async fn test_master_init() {
  Application::builder().add_plugin(DbPlugin).run().await.unwrap();
  let app = Application::global();

  // let (db_tx, db_rx) = mpsc::channel(1024);
  loop_master(app.clone()).await.unwrap()
}
