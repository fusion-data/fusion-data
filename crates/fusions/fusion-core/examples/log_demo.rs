//! 日志配置演示示例

use std::time::Duration;

use fusion_core::configuration::{LogConfig, LogWriterType};
use fusion_core::log::init_log;
use log::Level;

fn main() {
  println!("=== 日志配置演示 ===");

  // 演示同时输出到控制台和文件的配置
  println!("\n测试同时输出到控制台和文件:");
  let config = LogConfig {
    enable: true,
    with_target: true,
    with_file: true,
    with_thread_ids: true,
    with_thread_names: false,
    with_line_number: true,
    with_span_events: vec![],
    time_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
    log_level: Level::Debug.into(),
    log_targets: vec![],
    log_writer: LogWriterType::Both, // 同时输出到控制台和文件
    log_dir: "./target/demo_logs/".to_string(),
    log_name: Some("demo".to_string()),
    otel: Default::default(),
  };

  init_log(&config);

  log::info!("这是一条信息日志 - 应该同时出现在控制台和文件中");
  log::warn!("这是一条警告日志 - 应该同时出现在控制台和文件中");
  log::error!("这是一条错误日志 - 应该同时出现在控制台和文件中");

  println!("\n演示完成！请检查控制台输出和 ./target/demo_logs/demo.log 文件。");

  std::thread::sleep(Duration::from_secs(1)); // 确保日志被写入文件
}
