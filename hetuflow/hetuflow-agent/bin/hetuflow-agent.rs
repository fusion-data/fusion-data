use clap::{Arg, Command};
use hetuflow_agent::application::AgentApplication;
use hetuflow_agent::setting::AgentSetting;
use log::{error, info};
use logforth::append;
use std::path::PathBuf;
use std::process;
use tokio::signal;
use ultimate_core::DataError;

#[tokio::main]
async fn main() -> Result<(), DataError> {
    // 解析命令行参数
    let matches = Command::new("hetuflow-agent")
        .version(env!("CARGO_PKG_VERSION"))
        .about("HetuFlow Agent - 分布式任务执行代理")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("配置文件路径")
                .default_value("config.toml"),
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .long("log-level")
                .value_name("LEVEL")
                .help("日志级别 (trace, debug, info, warn, error)")
                .default_value("info"),
        )
        .arg(
            Arg::new("daemon")
                .short('d')
                .long("daemon")
                .help("以守护进程模式运行")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let log_level = matches.get_one::<String>("log-level").unwrap();
    let daemon_mode = matches.get_flag("daemon");

    // 初始化日志系统
    init_logging(log_level, daemon_mode)?;

    info!("HetuFlow Agent 启动中...");
    info!("配置文件: {}", config_path);
    info!("日志级别: {}", log_level);
    info!("守护进程模式: {}", daemon_mode);

    // 加载配置文件
    let config_path = PathBuf::from(config_path);
    let setting = match AgentSetting::from_file(&config_path) {
        Ok(setting) => {
            info!("配置文件加载成功");
            setting
        }
        Err(e) => {
            error!("配置文件加载失败: {}", e);
            process::exit(1);
        }
    };

    // 创建并启动 Agent 应用程序
    let mut app = match AgentApplication::from_setting(setting).await {
        Ok(app) => {
            info!("Agent 应用程序初始化成功");
            app
        }
        Err(e) => {
            error!("Agent 应用程序初始化失败: {}", e);
            process::exit(1);
        }
    };

    // 启动应用程序
    if let Err(e) = app.start().await {
        error!("Agent 应用程序启动失败: {}", e);
        process::exit(1);
    }

    info!("Agent 应用程序启动成功，Agent ID: {}", app.get_agent_id());
    info!("Agent 名称: {}", app.get_agent_name());

    // 等待关闭信号
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("接收到 Ctrl+C 信号，开始关闭...");
        }
        #[cfg(unix)]
        _ = async {
            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler");
            sigterm.recv().await;
        } => {
            info!("接收到 SIGTERM 信号，开始关闭...");
        }
    }

    // 优雅关闭
    info!("正在关闭 Agent 应用程序...");
    if let Err(e) = app.stop().await {
        error!("Agent 应用程序关闭失败: {}", e);
        process::exit(1);
    }

    info!("Agent 应用程序已成功关闭");
    Ok(())
}

/// 初始化日志系统
fn init_logging(log_level: &str, daemon_mode: bool) -> Result<(), DataError> {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => {
            eprintln!("无效的日志级别: {}", log_level);
            process::exit(1);
        }
    };

    let mut logger = logforth::Logger::new().max_level(level);

    if daemon_mode {
        // 守护进程模式：输出到文件
        logger = logger.dispatch(
            logforth::Dispatch::new()
                .filter(|metadata| metadata.target().starts_with("hetuflow"))
                .append(append::RollingFile::new(
                    "logs/hetuflow-agent.log",
                    append::rolling::Rotation::Daily,
                    append::rolling::Cleanup::KeepLogDays(7),
                ))
                .layout(logforth::layout::JsonLayout::default()),
        );
    } else {
        // 交互模式：输出到控制台
        logger = logger.dispatch(
            logforth::Dispatch::new()
                .filter(|metadata| metadata.target().starts_with("hetuflow"))
                .append(append::Stdout::new())
                .layout(logforth::layout::SimpleLayout::default()),
        );
    }

    logger.apply().map_err(|e| {
        DataError::SystemError(format!("日志系统初始化失败: {}", e))
    })?;

    Ok(())
}
