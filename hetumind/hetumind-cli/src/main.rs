use clap::Parser;
use hetumind_cli::{
  command::{Cli, Commands, WorkflowCommands},
  config::CliConfig,
  error::CliError,
  handlers::WorkflowHandler,
};

#[tokio::main]
async fn main() -> Result<(), CliError> {
  let cli = Cli::parse();

  // 加载配置
  let config = CliConfig::load()?;

  // 判断是否需要 API 访问
  let needs_api = matches!(
    &cli.command,
    Commands::Workflow(workflow_args) if matches!(
      workflow_args.command,
      WorkflowCommands::List(_) | WorkflowCommands::Run(_) |
      WorkflowCommands::Import(_) | WorkflowCommands::Export(_)
    )
  );

  // 对于需要 API 访问的命令，验证配置
  if needs_api && !config.is_valid() {
    eprintln!("错误: 配置无效，无法执行需要 API 访问的命令");
    eprintln!("请先配置 API endpoint 和 token");
    eprintln!("配置文件位置: {:?}", CliConfig::default_config_path()?);
    return Err(CliError::validation_error("配置验证失败"));
  }

  match &cli.command {
    Commands::Workflow(workflow_args) => {
      let handler = WorkflowHandler::create(config)?;

      match &workflow_args.command {
        WorkflowCommands::New(args) => handler.create_new(args).await,
        WorkflowCommands::List(args) => handler.list(args).await,
        WorkflowCommands::Validate(args) => handler.validate(args).await,
        WorkflowCommands::Run(args) => handler.run(args).await,
        WorkflowCommands::Import(args) => handler.import(args).await,
        WorkflowCommands::Export(args) => handler.export(args).await,
      }
    }
  }
}
