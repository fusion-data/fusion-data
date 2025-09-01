use std::fs;
use std::process::Command;
use tempfile::tempdir;

/// 集成测试辅助函数 - 运行CLI命令并设置独立的配置环境
fn run_cli_command(args: &[&str]) -> std::process::Output {
  let temp_dir = tempdir().unwrap();
  let config_dir = temp_dir.path().join(".hetumind");
  fs::create_dir_all(&config_dir).unwrap();

  let config_file = config_dir.join("config.toml");
  let config_content = r#"
[api]
endpoint = "http://localhost:8080"
token = "test-token-12345"
"#;

  fs::write(&config_file, config_content).unwrap();

  Command::new("cargo")
    .arg("run")
    .arg("--bin")
    .arg("hetumind-cli")
    .arg("--")
    .args(args)
    .env("GUIXU_CONFIG_PATH", config_file.to_str().unwrap())
    .current_dir(env!("CARGO_MANIFEST_DIR"))
    .output()
    .expect("Failed to execute CLI command")
}

/// 运行不需要API配置的CLI命令
fn run_cli_command_no_api(args: &[&str]) -> std::process::Output {
  let temp_dir = tempdir().unwrap();
  let config_file = temp_dir.path().join("config.toml");

  // 创建有效的配置文件，但token为空
  let config_content = r#"
[api]
endpoint = "http://localhost:8080"
token = ""
"#;
  fs::write(&config_file, config_content).unwrap();

  Command::new("cargo")
    .arg("run")
    .arg("--bin")
    .arg("hetumind-cli")
    .arg("--")
    .args(args)
    .env("GUIXU_CONFIG_PATH", config_file.to_str().unwrap())
    .current_dir(env!("CARGO_MANIFEST_DIR"))
    .output()
    .expect("Failed to execute CLI command")
}

/// 辅助函数：显示命令输出用于调试
fn debug_output(output: &std::process::Output, test_name: &str) {
  if !output.status.success() {
    println!("=== {} 失败 ===", test_name);
    println!("退出码: {:?}", output.status.code());
    println!("标准输出:");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("标准错误:");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    println!("==================");
  }
}

#[test]
fn test_cli_help_command() {
  let output = run_cli_command_no_api(&["--help"]);
  debug_output(&output, "help");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("Rust libraries of The fusions"));
  assert!(stdout.contains("管理工作流"));
}

#[test]
fn test_cli_workflow_help_command() {
  let output = run_cli_command_no_api(&["workflow", "--help"]);
  debug_output(&output, "workflow help");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("管理工作流"));
  assert!(stdout.contains("new"));
  assert!(stdout.contains("list"));
  assert!(stdout.contains("validate"));
  assert!(stdout.contains("run"));
  assert!(stdout.contains("import"));
  assert!(stdout.contains("export"));
}

#[test]
fn test_cli_workflow_new_command() {
  let temp_dir = tempdir().unwrap();
  let workflow_file = temp_dir.path().join("test-workflow.json");

  let output = run_cli_command_no_api(&[
    "workflow",
    "new",
    "--name",
    "integration-test-workflow",
    "--template",
    "default",
    "--output",
    workflow_file.to_str().unwrap(),
  ]);

  debug_output(&output, "workflow new");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("📝 正在创建新工作流"));
  assert!(stdout.contains("✅ 工作流文件已创建"));

  // 验证文件是否创建
  assert!(workflow_file.exists());

  // 验证文件内容
  let content = fs::read_to_string(&workflow_file).unwrap();
  assert!(content.contains("integration-test-workflow"));
  assert!(content.contains("\"status\": 1")); // Draft status
}

#[test]
fn test_cli_workflow_new_with_empty_template() {
  let temp_dir = tempdir().unwrap();
  let workflow_file = temp_dir.path().join("empty-workflow.json");

  let output = run_cli_command_no_api(&[
    "workflow",
    "new",
    "--name",
    "empty-test",
    "--template",
    "empty",
    "--output",
    workflow_file.to_str().unwrap(),
  ]);

  debug_output(&output, "workflow new empty");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("📝 正在创建新工作流"));
  assert!(stdout.contains("🎯 模板: empty"));

  // 验证文件是否创建
  assert!(workflow_file.exists());

  // 验证文件内容
  let content = fs::read_to_string(&workflow_file).unwrap();
  assert!(content.contains("empty-test"));
  assert!(content.contains("\"nodes\": []"));
  assert!(content.contains("\"connections\": []"));
}

#[test]
fn test_cli_workflow_new_with_invalid_template() {
  let temp_dir = tempdir().unwrap();
  let workflow_file = temp_dir.path().join("invalid-workflow.json");

  let output = run_cli_command_no_api(&[
    "workflow",
    "new",
    "--name",
    "invalid-test",
    "--template",
    "invalid-template",
    "--output",
    workflow_file.to_str().unwrap(),
  ]);

  debug_output(&output, "workflow new invalid");
  assert!(!output.status.success());

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(stderr.contains("未知的模板") || stderr.contains("validation"));

  // 验证文件未创建
  assert!(!workflow_file.exists());
}

#[test]
fn test_cli_workflow_validate_valid_file() {
  let temp_dir = tempdir().unwrap();
  let workflow_file = temp_dir.path().join("valid-workflow.json");

  // 首先创建一个有效的工作流文件
  let create_output = run_cli_command_no_api(&[
    "workflow",
    "new",
    "--name",
    "validation-test",
    "--template",
    "default",
    "--output",
    workflow_file.to_str().unwrap(),
  ]);

  // 确保创建成功
  assert!(create_output.status.success());
  assert!(workflow_file.exists());

  // 然后验证该文件
  let output = run_cli_command_no_api(&["workflow", "validate", workflow_file.to_str().unwrap()]);

  debug_output(&output, "workflow validate valid");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("🔍 正在验证工作流文件"));
  assert!(stdout.contains("✅ 工作流文件加载成功"));
  assert!(stdout.contains("🎉 工作流验证完成"));
}

#[test]
fn test_cli_workflow_validate_nonexistent_file() {
  let output = run_cli_command_no_api(&["workflow", "validate", "/non/existent/file.json"]);

  debug_output(&output, "workflow validate nonexistent");
  assert!(!output.status.success());

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(stderr.contains("文件不存在") || stderr.contains("validation"));
}

#[test]
fn test_cli_workflow_validate_invalid_json() {
  let temp_dir = tempdir().unwrap();
  let invalid_file = temp_dir.path().join("invalid.json");

  // 创建无效的 JSON 文件
  fs::write(&invalid_file, "{ invalid json content }").unwrap();

  let output = run_cli_command_no_api(&["workflow", "validate", invalid_file.to_str().unwrap()]);

  debug_output(&output, "workflow validate invalid json");
  assert!(!output.status.success());

  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(stderr.contains("解析工作流文件失败") || stderr.contains("validation"));
}

#[test]
fn test_cli_workflow_list_command() {
  let output = run_cli_command(&["workflow", "list"]);

  debug_output(&output, "workflow list");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("🔍 正在查询工作流列表"));
}

#[test]
fn test_cli_workflow_run_with_id() {
  let output = run_cli_command(&["workflow", "run", "test-workflow-id"]);

  debug_output(&output, "workflow run with id");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("🚀 正在执行工作流"));
  assert!(stdout.contains("test-workflow-id"));
}

#[test]
fn test_cli_workflow_run_with_file() {
  let temp_dir = tempdir().unwrap();
  let workflow_file = temp_dir.path().join("run-test-workflow.json");

  // 创建工作流文件
  let create_output = run_cli_command_no_api(&[
    "workflow",
    "new",
    "--name",
    "run-test",
    "--template",
    "default",
    "--output",
    workflow_file.to_str().unwrap(),
  ]);

  // 确保创建成功
  assert!(create_output.status.success());
  assert!(workflow_file.exists());

  let output = run_cli_command(&["workflow", "run", workflow_file.to_str().unwrap()]);

  debug_output(&output, "workflow run with file");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("🚀 正在执行工作流"));
  assert!(stdout.contains("📁 检测到文件路径"));
}

#[test]
fn test_cli_workflow_import_command() {
  let temp_dir = tempdir().unwrap();
  let workflow_file = temp_dir.path().join("import-test-workflow.json");

  // 创建工作流文件
  let create_output = run_cli_command_no_api(&[
    "workflow",
    "new",
    "--name",
    "import-test",
    "--template",
    "default",
    "--output",
    workflow_file.to_str().unwrap(),
  ]);

  // 确保创建成功
  assert!(create_output.status.success());
  assert!(workflow_file.exists());

  let output = run_cli_command(&["workflow", "import", workflow_file.to_str().unwrap()]);

  debug_output(&output, "workflow import");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("📤 正在导入工作流文件"));
}

#[test]
fn test_cli_workflow_export_command() {
  let output = run_cli_command(&["workflow", "export", "test-workflow-id"]);

  debug_output(&output, "workflow export");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("📥 正在导出工作流"));
}

#[test]
fn test_cli_workflow_new_default_output_path() {
  let temp_dir = tempdir().unwrap();
  let expected_file = temp_dir.path().join("default-output-test.json");

  let output = run_cli_command_no_api(&[
    "workflow",
    "new",
    "--name",
    "default-output-test",
    "--template",
    "default",
    "--output",
    expected_file.to_str().unwrap(),
  ]);

  debug_output(&output, "workflow new default output");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("📝 正在创建新工作流"));
  assert!(stdout.contains("✅ 工作流文件已创建"));

  // 验证文件是否创建
  assert!(expected_file.exists());
}

#[test]
fn test_cli_workflow_run_with_sync_flag() {
  let output = run_cli_command(&["workflow", "run", "test-workflow-id", "--sync"]);

  debug_output(&output, "workflow run with sync");
  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("🚀 正在执行工作流"));
  assert!(stdout.contains("⏳ 同步等待执行完成"));
}
