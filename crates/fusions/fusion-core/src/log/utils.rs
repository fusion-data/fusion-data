use std::path::Path;

use log::{Level, LevelFilter};
use logforth::append::Stdout;
use logforth::append::rolling_file::{RollingFileBuilder, Rotation};
use logforth::filter::EnvFilter;
use logforth::layout::TextLayout;
use logforth::{DropGuard, LoggerBuilder};

use crate::configuration::LogSetting;
use std::sync::Once;

pub fn get_trace_id() -> Option<String> {
  // TODO:
  None
}

pub fn init_log(conf: &LogSetting) {
  // 如果日志未启用，则不进行配置
  if !conf.enable() {
    return;
  }

  // 将 LogLevel 转换为 LevelFilter
  let level_filter = match conf.log_level.0 {
    Level::Error => LevelFilter::Error,
    Level::Warn => LevelFilter::Warn,
    Level::Info => LevelFilter::Info,
    Level::Debug => LevelFilter::Debug,
    Level::Trace => LevelFilter::Trace,
  };

  let mut builder = logforth::builder();

  // 根据 log_writer 配置不同的输出目标
  match conf.log_writer {
    crate::configuration::LogWriterType::Stdout => {
      builder = dispatch_stdout(conf, level_filter, builder);
    }
    crate::configuration::LogWriterType::File => {
      builder = dispatch_file(conf, level_filter, builder);
    }
    crate::configuration::LogWriterType::Both => {
      builder = dispatch_stdout(conf, level_filter, builder);
      builder = dispatch_file(conf, level_filter, builder);
    }
  }

  // 应用配置
  builder.apply();

  println!("日志系统初始化完成，级别: {}, 输出: {:?}", conf.log_level, conf.log_writer);
}

// Store the guard in a static variable to keep the file appender alive
static mut LOG_GUARD: Option<DropGuard> = None;
static INIT: Once = Once::new();

fn dispatch_file(conf: &LogSetting, level_filter: LevelFilter, builder: LoggerBuilder) -> LoggerBuilder {
  let log_name = conf.log_name.as_deref().unwrap_or("app.log");
  let log_path = Path::new(&conf.log_dir);
  let _ = std::fs::create_dir_all(log_path);

  let (file_appender, guard) = RollingFileBuilder::new(log_path)
    .filename_prefix(log_name)
    .filename_suffix("log")
    .rotation(Rotation::Daily)
    .max_log_files(30)
    .layout(build_text_layout(conf).no_color())
    .build()
    .expect("Failed to create log file appender");

  // Initialize the guard
  unsafe {
    INIT.call_once(|| {
      LOG_GUARD = Some(guard);
    });
  }

  let env_filter2 = build_env_filter(conf, level_filter);
  builder.dispatch(|d| d.filter(env_filter2).append(file_appender))
}

fn dispatch_stdout(conf: &LogSetting, level_filter: LevelFilter, builder: LoggerBuilder) -> LoggerBuilder {
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
  EnvFilter::from_default_env_or(&filter_str)
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

/// 根据配置构建文本布局
fn build_text_layout(_conf: &LogSetting) -> TextLayout {
  // let tz = jiff::tz::TimeZone::fixed(jiff::tz::offset(8));
  TextLayout::default() //.timezone(tz)
}
