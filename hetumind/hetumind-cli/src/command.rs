use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Hetumind CLI - 强大而灵活的工作流引擎命令行工具
///
/// 用于管理和执行 Hetumind 工作流，支持本地开发和远程部署。
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
  /// 管理工作流 - 创建、列出、验证、运行、导入和导出工作流
  #[command(alias = "wf")]
  Workflow(WorkflowArgs),
}

#[derive(Args, Debug)]
pub struct WorkflowArgs {
  #[command(subcommand)]
  pub command: WorkflowCommands,
}

#[derive(Subcommand, Debug)]
pub enum WorkflowCommands {
  /// 从模板创建新的工作流定义文件
  New(NewWorkflow),
  /// 列出服务器上所有可用的工作流，按创建时间降序排列
  List(ListWorkflows),
  /// 验证本地工作流文件的语法和结构
  Validate(ValidateWorkflow),
  /// 从文件或通过ID运行工作流
  Run(RunWorkflow),
  /// 将本地工作流文件导入到服务器
  Import(ImportWorkflow),
  /// 从服务器导出工作流定义
  Export(ExportWorkflow),
}

/// 创建新工作流的参数
#[derive(Args, Debug)]
pub struct NewWorkflow {
  /// 新工作流的名称
  #[arg(short, long, help = "新工作流的名称")]
  pub name: String,

  /// 使用的工作流模板
  #[arg(short, long, default_value = "default", help = "工作流模板类型：default（默认模板）或 empty（空模板）")]
  pub template: String,

  /// 输出文件路径
  #[arg(short, long, help = "输出工作流文件的路径，默认为 <名称>.json")]
  pub output: Option<PathBuf>,
}

/// 列出工作流的参数
#[derive(Args, Debug)]
pub struct ListWorkflows {
  /// 按状态过滤工作流
  #[arg(short, long, help = "根据状态过滤（如：active、draft、archived）")]
  pub status: Option<String>,

  /// 返回的最大工作流数量
  #[arg(short, long, default_value_t = 20, help = "返回的最大工作流数量")]
  pub limit: u32,
}

/// 验证工作流的参数
#[derive(Args, Debug)]
pub struct ValidateWorkflow {
  /// 要验证的工作流文件路径
  #[arg(value_name = "FILE", help = "要验证的工作流文件路径")]
  pub path: PathBuf,
}

/// 运行工作流的参数
#[derive(Args, Debug)]
pub struct RunWorkflow {
  /// 工作流ID或文件路径
  #[arg(value_name = "ID_OR_FILE", help = "要执行的工作流ID或本地文件路径")]
  pub id_or_file: String,

  /// 输入数据文件路径
  #[arg(short, long, help = "包含输入数据的JSON文件路径")]
  pub input: Option<PathBuf>,

  /// 同步执行模式
  #[arg(long, help = "同步等待执行完成，而不是立即返回执行ID")]
  pub sync: bool,
}

/// 导入工作流的参数
#[derive(Args, Debug)]
pub struct ImportWorkflow {
  /// 要导入的工作流文件路径
  #[arg(value_name = "FILE", help = "要导入到服务器的工作流文件路径")]
  pub path: PathBuf,
}

/// 导出工作流的参数
#[derive(Args, Debug)]
pub struct ExportWorkflow {
  /// 要导出的工作流ID
  #[arg(help = "要从服务器导出的工作流ID")]
  pub id: String,

  /// 输出格式
  #[arg(short, long, default_value = "json", help = "输出格式（json 或 yaml）")]
  pub format: String,

  /// 输出文件路径
  #[arg(short, long, help = "输出文件路径，未指定时输出到标准输出")]
  pub output: Option<PathBuf>,
}
