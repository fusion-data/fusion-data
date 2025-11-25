use std::num::NonZeroUsize;
use std::path::Path;

use log::{Level, LevelFilter, info};
use logforth::append::Stdout;
use logforth::append::file::FileBuilder;
use logforth::filter::EnvFilter;
use logforth::filter::env_filter::EnvFilterBuilder;
use logforth::layout::TextLayout;
use logforth::starter_log::LogStarterBuilder;

use crate::configuration::{LogSetting, LogWriterType};

pub fn get_trace_id() -> Option<String> {
  // TODO:
  None
}

pub fn init_log(conf: &LogSetting) {
  // If the log is not enabled, do not enable it.
  if !conf.enable() {
    return;
  }

  // Convert LogLevel to LevelFilter
  let level_filter = match conf.log_level.0 {
    Level::Error => LevelFilter::Error,
    Level::Warn => LevelFilter::Warn,
    Level::Info => LevelFilter::Info,
    Level::Debug => LevelFilter::Debug,
    Level::Trace => LevelFilter::Trace,
  };

  let mut builder = logforth::starter_log::builder();

  // 根据 log_writer 配置不同的输出目标

  for log_writer in &conf.log_writers {
    builder = match log_writer {
      LogWriterType::Stdout => dispatch_stdout(conf, level_filter, builder),
      LogWriterType::File => dispatch_file(conf, level_filter, builder),
    };
  }

  // 应用配置
  builder.apply();

  info!("Log system initialization completed, level: {}, writer: {:?}", conf.log_level, conf.log_writers);
}

fn dispatch_file(conf: &LogSetting, level_filter: LevelFilter, builder: LogStarterBuilder) -> LogStarterBuilder {
  let log_name = conf.log_name.as_deref().unwrap_or("app.log");
  let log_path = Path::new(&conf.log_dir);
  let _ = std::fs::create_dir_all(log_path);

  let file_appender = FileBuilder::new(log_path, log_name)
    .rollover_daily()
    .max_log_files(NonZeroUsize::new(30).expect("Init NonZereUsize 30 failed"))
    .layout(build_text_layout(conf).no_color())
    .build()
    .expect("Failed to create log file appender");

  let env_filter2 = build_env_filter(conf, level_filter);
  builder.dispatch(|d| d.filter(env_filter2).append(file_appender))
}

fn dispatch_stdout(conf: &LogSetting, level_filter: LevelFilter, builder: LogStarterBuilder) -> LogStarterBuilder {
  let env_filter = build_env_filter(conf, level_filter);
  let layout = build_text_layout(conf);
  builder.dispatch(|d| d.filter(env_filter).append(Stdout::default().with_layout(layout)))
}

/// 构建 EnvFilter，支持 log_targets 配置
fn build_env_filter(conf: &LogSetting, default_level: LevelFilter) -> EnvFilter {
  let mut filter_parts = Vec::new();

  // 添加默认级别
  let default_level_str = level_filter_to_string(default_level);

  // 处理 log_targets 配置
  for target in &conf.log_targets {
    if !target.is_empty() {
      filter_parts.push(target.clone());
    }
  }

  // 如果没有配置 log_targets，使用默认级别
  if filter_parts.is_empty() {
    filter_parts.push(default_level_str);
  }

  // 组合过滤器字符串
  let filter_str = filter_parts.join(",");

  // 构建 EnvFilter
  EnvFilterBuilder::from_default_env_or(filter_str).build()
}

/// 将 LevelFilter 转换为字符串
fn level_filter_to_string(level: LevelFilter) -> String {
  match level {
    LevelFilter::Off => "off".to_string(),
    LevelFilter::Error => "error".to_string(),
    LevelFilter::Warn => "warn".to_string(),
    LevelFilter::Info => "info".to_string(),
    LevelFilter::Debug => "debug".to_string(),
    LevelFilter::Trace => "trace".to_string(),
  }
}

fn build_text_layout(_conf: &LogSetting) -> TextLayout {
  TextLayout::default()
}
