use std::path::PathBuf;

use fusions::common::ahash::HashMap;
use hetumind_core::workflow::{Workflow, WorkflowId, WorkflowStatus};

use crate::{
  api::ApiClient,
  command::{ExportWorkflow, ImportWorkflow, ListWorkflows, NewWorkflow, RunWorkflow, ValidateWorkflow},
  config::CliConfig,
  error::{CliError, CliResult},
};

/// 工作流相关命令处理器
pub struct WorkflowHandler {
  api_client: ApiClient,
}

impl WorkflowHandler {
  /// 创建新的工作流处理器
  pub fn create(config: CliConfig) -> CliResult<Self> {
    let api_client = ApiClient::new(config)?;
    Ok(Self { api_client })
  }

  /// 列出工作流
  pub async fn list(&self, args: &ListWorkflows) -> CliResult<()> {
    println!("🔍 正在查询工作流列表...");
    println!("⚠️  列表功能需要等待 API 客户端完善后实现");
    println!("参数: status={:?}, limit={}", args.status, args.limit);
    Ok(())
  }

  /// 验证工作流文件
  pub async fn validate(&self, args: &ValidateWorkflow) -> CliResult<()> {
    println!("🔍 正在验证工作流文件: {:?}", args.path);

    // 检查文件是否存在
    if !args.path.exists() {
      return Err(CliError::validation_error(format!("文件不存在: {:?}", args.path)));
    }

    // 加载工作流文件
    println!("📄 正在加载工作流文件...");
    let workflow = self.api_client.load_workflow_from_file(&args.path).await?;
    println!("✅ 工作流文件加载成功: {}", workflow.name);

    // 本地基础验证
    println!("🔍 执行本地验证...");

    // 验证连接性
    // TODO 调用远程 API 验证工作流连接性

    println!("🎉 工作流验证完成！");
    Ok(())
  }

  /// 运行工作流
  pub async fn run(&self, args: &RunWorkflow) -> CliResult<()> {
    println!("🚀 正在执行工作流: {}", args.id_or_file);

    // 解析输入数据
    let _input_data = if let Some(input_file) = &args.input {
      println!("📄 加载输入数据文件: {:?}", input_file);
      Some(self.api_client.load_input_data_from_file(input_file).await?)
    } else {
      None
    };

    // 判断是 ID 还是文件路径
    if args.id_or_file.contains('/') || args.id_or_file.ends_with(".json") {
      // 看起来是文件路径，需要先导入
      println!("📁 检测到文件路径，正在导入工作流...");
      let path = PathBuf::from(&args.id_or_file);
      let workflow = self.api_client.load_workflow_from_file(&path).await?;
      println!("✅ 工作流文件加载成功: {}", workflow.name);
      println!("⚠️  执行功能需要等待 API 客户端完善后实现");
    } else {
      // 应该是 workflow ID
      println!("🔍 使用工作流 ID: {}", args.id_or_file);
      println!("⚠️  执行功能需要等待 API 客户端完善后实现");
    }

    if args.sync {
      println!("⏳ 同步等待执行完成...");
      println!("⚠️  同步等待功能尚未实现");
    }

    Ok(())
  }

  /// 创建新的工作流文件
  pub async fn create_new(&self, args: &NewWorkflow) -> CliResult<()> {
    println!("📝 正在创建新工作流: {}", args.name);

    // 确定输出路径
    let output_path = args.output.clone().unwrap_or_else(|| PathBuf::from(format!("{}.json", args.name)));

    // 创建基础工作流模板
    let workflow = self.create_simple_workflow_template(&args.name, &args.template)?;

    // 保存到文件
    self.api_client.save_workflow_to_file(&workflow, &output_path).await?;

    println!("✅ 工作流文件已创建: {:?}", output_path);
    println!("📋 工作流 ID: {}", workflow.id);
    println!("🎯 模板: {}", args.template);

    Ok(())
  }

  /// 导入工作流到服务端
  pub async fn import(&self, args: &ImportWorkflow) -> CliResult<()> {
    println!("📤 正在导入工作流文件: {:?}", args.path);

    // 加载工作流文件
    let workflow = self.api_client.load_workflow_from_file(&args.path).await?;
    println!("✅ 工作流文件加载成功: {}", workflow.name);

    println!("⚠️  导入功能需要等待 API 客户端完善后实现");

    Ok(())
  }

  /// 导出工作流到文件
  pub async fn export(&self, args: &ExportWorkflow) -> CliResult<()> {
    println!("📥 正在导出工作流: {}", args.id);

    println!("⚠️  导出功能需要等待 API 客户端完善后实现");

    Ok(())
  }

  /// 创建简单的工作流模板
  fn create_simple_workflow_template(&self, name: &str, template: &str) -> CliResult<Workflow> {
    use hetumind_core::workflow::{ErrorHandlingStrategy, ExecutionMode, PinData, WorkflowMeta, WorkflowSettings};

    let workflow_id = WorkflowId::now_v7();

    match template {
      "default" | "empty" => {
        Ok(Workflow {
          id: workflow_id,
          name: name.to_string(),
          status: WorkflowStatus::Draft,
          version: None,
          settings: WorkflowSettings {
            execution_timeout: Some(300), // 5分钟
            error_handling: Some(ErrorHandlingStrategy::StopOnFirstError),
            execution_mode: Some(ExecutionMode::Local),
            remark: Some(format!("使用 {template} 模板创建")),
          },
          meta: WorkflowMeta::default(),
          nodes: Vec::new(),
          connections: HashMap::default(),
          pin_data: PinData::default(),
          static_data: None,
        })
      }
      _ => Err(CliError::validation_error(format!("未知的模板: {template}"))),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{ApiConfig, CliConfig};
  use std::io::Write;
  use std::path::PathBuf;
  use tempfile::{NamedTempFile, tempdir};

  fn create_test_config() -> CliConfig {
    CliConfig { api: ApiConfig { endpoint: "http://localhost:8080".to_string(), token: "test-token".to_string() } }
  }

  fn create_test_handler() -> WorkflowHandler {
    let config = create_test_config();
    WorkflowHandler::create(config).unwrap()
  }

  #[test]
  fn test_workflow_handler_creation() {
    let config = create_test_config();
    let handler = WorkflowHandler::create(config);
    assert!(handler.is_ok());
  }

  #[tokio::test]
  async fn test_create_new_workflow_with_default_template() {
    let handler = create_test_handler();
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("test-workflow.json");

    let args = NewWorkflow {
      name: "test-workflow".to_string(),
      template: "default".to_string(),
      output: Some(output_path.clone()),
    };

    let result = handler.create_new(&args).await;
    assert!(result.is_ok());

    // 验证文件是否创建
    assert!(output_path.exists());

    // 验证文件内容
    let content = std::fs::read_to_string(&output_path).unwrap();
    let workflow: Workflow = serde_json::from_str(&content).unwrap();
    assert_eq!(workflow.name, "test-workflow");
    assert_eq!(workflow.status, WorkflowStatus::Draft);
  }

  #[tokio::test]
  async fn test_create_new_workflow_with_empty_template() {
    let handler = create_test_handler();
    let temp_dir = tempdir().unwrap();
    let output_path = temp_dir.path().join("empty-workflow.json");

    let args = NewWorkflow {
      name: "empty-workflow".to_string(),
      template: "empty".to_string(),
      output: Some(output_path.clone()),
    };

    let result = handler.create_new(&args).await;
    assert!(result.is_ok());

    // 验证文件是否创建
    assert!(output_path.exists());

    // 验证文件内容
    let content = std::fs::read_to_string(&output_path).unwrap();
    let workflow: Workflow = serde_json::from_str(&content).unwrap();
    assert_eq!(workflow.name, "empty-workflow");
    assert!(workflow.nodes.is_empty());
    assert!(workflow.connections.is_empty());
  }

  #[tokio::test]
  async fn test_create_new_workflow_with_invalid_template() {
    let handler = create_test_handler();

    let args =
      NewWorkflow { name: "test-workflow".to_string(), template: "invalid-template".to_string(), output: None };

    let result = handler.create_new(&args).await;
    assert!(result.is_err());

    if let Err(CliError::ValidationError { message }) = result {
      assert!(message.contains("未知的模板"));
    } else {
      panic!("Expected validation error");
    }
  }

  #[tokio::test]
  async fn test_create_new_workflow_with_default_output_path() {
    let handler = create_test_handler();

    let args = NewWorkflow {
      name: "auto-path".to_string(),
      template: "default".to_string(),
      output: None, // 使用默认路径
    };

    let result = handler.create_new(&args).await;
    assert!(result.is_ok());

    // 检查默认路径是否创建了文件
    let expected_path = PathBuf::from("auto-path.json");
    assert!(expected_path.exists());

    // 清理测试文件
    std::fs::remove_file(expected_path).ok();
  }

  #[tokio::test]
  async fn test_validate_workflow_success() {
    let handler = create_test_handler();

    // 创建有效的工作流文件
    let mut temp_file = NamedTempFile::new().unwrap();
    let workflow_json = r#"
        {
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Valid Workflow",
            "status": 1,
            "version": null,
            "settings": {
                "execution_timeout": null,
                "error_handling": "StopOnFirstError",
                "execution_mode": 1,
                "remark": null
            },
            "meta": {
                "credentials_setup_completed": false
            },
            "nodes": [],
            "connections": {},
            "pin_data": {
                "data": {}
            },
            "static_data": null
        }
        "#;

    temp_file.write_all(workflow_json.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let args = ValidateWorkflow { path: temp_path };

    let result = handler.validate(&args).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_validate_workflow_file_not_exists() {
    let handler = create_test_handler();

    let args = ValidateWorkflow { path: PathBuf::from("/non/existent/file.json") };

    let result = handler.validate(&args).await;
    assert!(result.is_err());

    if let Err(CliError::ValidationError { message }) = result {
      assert!(message.contains("文件不存在"));
    } else {
      panic!("Expected validation error");
    }
  }

  #[tokio::test]
  async fn test_validate_workflow_invalid_json() {
    let handler = create_test_handler();

    // 创建无效的 JSON 文件
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"invalid json content").unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let args = ValidateWorkflow { path: temp_path };

    let result = handler.validate(&args).await;
    assert!(result.is_err());

    if let Err(CliError::ValidationError { message }) = result {
      assert!(message.contains("解析工作流文件失败"));
    } else {
      panic!("Expected validation error");
    }
  }

  #[tokio::test]
  async fn test_run_workflow_with_file_path() {
    let handler = create_test_handler();

    // 创建测试工作流文件
    let mut temp_file = NamedTempFile::new().unwrap();
    let workflow_json = r#"
        {
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Test Run Workflow",
            "status": 1,
            "version": null,
            "settings": {
                "execution_timeout": null,
                "error_handling": "StopOnFirstError",
                "execution_mode": 1,
                "remark": null
            },
            "meta": {
                "credentials_setup_completed": false
            },
            "nodes": [],
            "connections": {},
            "pin_data": {
                "data": {}
            },
            "static_data": null
        }
        "#;

    temp_file.write_all(workflow_json.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_string_lossy().to_string();

    let args = RunWorkflow { id_or_file: temp_path, input: None, sync: false };

    let result = handler.run(&args).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_run_workflow_with_id() {
    let handler = create_test_handler();

    let args = RunWorkflow { id_or_file: "12345678-1234-1234-1234-123456789012".to_string(), input: None, sync: false };

    let result = handler.run(&args).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_run_workflow_with_input_file() {
    let handler = create_test_handler();

    // 创建输入数据文件
    let mut input_file = NamedTempFile::new().unwrap();
    input_file.write_all(br#"{"key": "value", "number": 42}"#).unwrap();
    let input_path = input_file.path().to_path_buf();

    let args = RunWorkflow {
      id_or_file: "12345678-1234-1234-1234-123456789012".to_string(),
      input: Some(input_path),
      sync: true,
    };

    let result = handler.run(&args).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_list_workflows() {
    let handler = create_test_handler();

    let args = ListWorkflows { status: Some("active".to_string()), limit: 10 };

    let result = handler.list(&args).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_import_workflow() {
    let handler = create_test_handler();

    // 创建测试工作流文件
    let mut temp_file = NamedTempFile::new().unwrap();
    let workflow_json = r#"
        {
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Import Test Workflow",
            "status": 1,
            "version": null,
            "settings": {
                "execution_timeout": null,
                "error_handling": "StopOnFirstError",
                "execution_mode": 1,
                "remark": null
            },
            "meta": {
                "credentials_setup_completed": false
            },
            "nodes": [],
            "connections": {},
            "pin_data": {
                "data": {}
            },
            "static_data": null
        }
        "#;

    temp_file.write_all(workflow_json.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let args = ImportWorkflow { path: temp_path };

    let result = handler.import(&args).await;
    assert!(result.is_ok());
  }

  #[tokio::test]
  async fn test_export_workflow() {
    let handler = create_test_handler();

    let args = ExportWorkflow {
      id: "12345678-1234-1234-1234-123456789012".to_string(),
      format: "json".to_string(),
      output: None,
    };

    let result = handler.export(&args).await;
    assert!(result.is_ok());
  }

  #[test]
  fn test_create_simple_workflow_template_default() {
    let handler = create_test_handler();

    let result = handler.create_simple_workflow_template("test", "default");
    assert!(result.is_ok());

    let workflow = result.unwrap();
    assert_eq!(workflow.name, "test");
    assert_eq!(workflow.status, WorkflowStatus::Draft);
    assert!(workflow.nodes.is_empty());
    assert!(workflow.connections.is_empty());
  }

  #[test]
  fn test_create_simple_workflow_template_empty() {
    let handler = create_test_handler();

    let result = handler.create_simple_workflow_template("empty-test", "empty");
    assert!(result.is_ok());

    let workflow = result.unwrap();
    assert_eq!(workflow.name, "empty-test");
    assert!(workflow.nodes.is_empty());
    assert!(workflow.connections.is_empty());
  }

  #[test]
  fn test_create_simple_workflow_template_invalid() {
    let handler = create_test_handler();

    let result = handler.create_simple_workflow_template("test", "invalid");
    assert!(result.is_err());

    if let Err(CliError::ValidationError { message }) = result {
      assert!(message.contains("未知的模板"));
    } else {
      panic!("Expected validation error");
    }
  }
}
