use std::path::PathBuf;

use hetumind_core::workflow::{ExecutionForQuery, ParameterMap, WorkflowForQuery, WorkflowForUpdate};
use reqwest::{Client, Method, Response, header};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
  config::CliConfig,
  error::{CliError, CliResult},
};

use super::models::*;

/// Hetumind API 客户端
#[derive(Debug, Clone)]
pub struct ApiClient {
  client: Client,
  config: CliConfig,
}

impl ApiClient {
  /// 创建新的 API 客户端
  pub fn new(config: CliConfig) -> CliResult<Self> {
    let mut headers = header::HeaderMap::new();

    // 添加认证头
    if !config.api.token.is_empty() {
      let auth_value = header::HeaderValue::from_str(&config.auth_header())
        .map_err(|e| CliError::config_error(format!("无效的认证令牌: {}", e)))?;
      headers.insert(header::AUTHORIZATION, auth_value);
    }

    // 添加内容类型头
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));

    let client = Client::builder()
      .default_headers(headers)
      .build()
      .map_err(|e| CliError::api_error(format!("创建 HTTP 客户端失败: {}", e)))?;

    Ok(Self { client, config })
  }

  /// 发送 HTTP 请求
  async fn request<T: Serialize, R: DeserializeOwned>(
    &self,
    method: Method,
    path: &str,
    body: Option<&T>,
  ) -> CliResult<R> {
    let url = self.config.api_url(path);

    let mut request_builder = self.client.request(method, &url);

    if let Some(body) = body {
      let json_body = serde_json::to_string(body)?;
      request_builder = request_builder.body(json_body);
    }

    let response = request_builder.send().await.map_err(|e| CliError::api_error(format!("HTTP 请求失败: {}", e)))?;

    self.handle_response(response).await
  }

  /// 处理响应
  async fn handle_response<R: DeserializeOwned>(&self, response: Response) -> CliResult<R> {
    let status = response.status();
    let response_text = response.text().await.map_err(|e| CliError::api_error(format!("读取响应失败: {}", e)))?;

    if status.is_success() {
      serde_json::from_str(&response_text).map_err(|e| CliError::api_error(format!("解析响应失败: {}", e)))
    } else {
      // 尝试解析错误响应
      if let Ok(api_error) = serde_json::from_str::<ApiError>(&response_text) {
        Err(CliError::api_error(format!("API 错误: {}", api_error.message)))
      } else {
        Err(CliError::api_error(format!("HTTP {} 错误: {}", status.as_u16(), response_text)))
      }
    }
  }

  // --- 工作流相关 API ---

  /// 创建工作流
  pub async fn create_workflow(&self, workflow: &Workflow) -> CliResult<WorkflowId> {
    self.request(Method::POST, "workflows", Some(workflow)).await
  }

  /// 查询工作流列表
  pub async fn query_workflows(&self, query: &WorkflowForQuery) -> CliResult<WorkflowListResponse> {
    self.request(Method::POST, "workflows/query", Some(query)).await
  }

  /// 验证工作流
  pub async fn validate_workflow(&self, request: &ValidateWorkflowRequest) -> CliResult<ValidateWorkflowResponse> {
    self.request(Method::POST, "workflows/validate", Some(request)).await
  }

  /// 获取工作流详情
  pub async fn get_workflow(&self, workflow_id: &WorkflowId) -> CliResult<Workflow> {
    let path = format!("workflows/{}", workflow_id);
    self.request::<(), _>(Method::GET, &path, None).await
  }

  /// 更新工作流
  pub async fn update_workflow(&self, workflow_id: &WorkflowId, update: &WorkflowForUpdate) -> CliResult<WorkflowId> {
    let path = format!("workflows/{}", workflow_id);
    self.request(Method::PUT, &path, Some(update)).await
  }

  /// 删除工作流
  pub async fn delete_workflow(&self, workflow_id: &WorkflowId) -> CliResult<()> {
    let path = format!("workflows/{}", workflow_id);
    self.request::<(), _>(Method::DELETE, &path, None).await
  }

  /// 执行工作流
  pub async fn execute_workflow(
    &self,
    workflow_id: &WorkflowId,
    request: &ExecuteWorkflowRequest,
  ) -> CliResult<ExecutionIdResponse> {
    let path = format!("workflows/{}/execute", workflow_id);
    self.request(Method::POST, &path, Some(request)).await
  }

  /// 激活工作流
  pub async fn activate_workflow(&self, workflow_id: &WorkflowId) -> CliResult<()> {
    let path = format!("workflows/{}/activate", workflow_id);
    self.request::<(), _>(Method::POST, &path, None).await
  }

  /// 停用工作流
  pub async fn deactivate_workflow(&self, workflow_id: &WorkflowId) -> CliResult<()> {
    let path = format!("workflows/{}/deactivate", workflow_id);
    self.request::<(), _>(Method::POST, &path, None).await
  }

  /// 复制工作流
  pub async fn duplicate_workflow(&self, workflow_id: &WorkflowId) -> CliResult<WorkflowId> {
    let path = format!("workflows/{}/duplicate", workflow_id);
    self.request::<(), _>(Method::POST, &path, None).await
  }

  // --- 执行相关 API ---

  /// 查询执行列表
  pub async fn query_executions(&self, query: &ExecutionForQuery) -> CliResult<ExecutionListResponse> {
    self.request(Method::POST, "executions/query", Some(query)).await
  }

  /// 获取执行详情
  pub async fn get_execution(&self, execution_id: &ExecutionId) -> CliResult<Execution> {
    let path = format!("executions/{}", execution_id);
    self.request::<(), _>(Method::GET, &path, None).await
  }

  /// 取消执行
  pub async fn cancel_execution(&self, execution_id: &ExecutionId) -> CliResult<()> {
    let path = format!("executions/{}/cancel", execution_id);
    self.request::<(), _>(Method::POST, &path, None).await
  }

  /// 重试执行
  pub async fn retry_execution(&self, execution_id: &ExecutionId) -> CliResult<()> {
    let path = format!("executions/{}/retry", execution_id);
    self.request::<(), _>(Method::POST, &path, None).await
  }

  /// 获取执行日志
  pub async fn get_execution_logs(&self, execution_id: &ExecutionId) -> CliResult<ExecutionLogsResponse> {
    let path = format!("executions/{}/logs", execution_id);
    self.request::<(), _>(Method::GET, &path, None).await
  }

  // --- 文件操作辅助方法 ---

  /// 从本地文件加载工作流
  pub async fn load_workflow_from_file(&self, path: &PathBuf) -> CliResult<Workflow> {
    let content = tokio::fs::read_to_string(path).await.map_err(|e| CliError::io_error(path.clone(), e))?;

    serde_json::from_str(&content).map_err(|e| CliError::validation_error(format!("解析工作流文件失败: {}", e)))
  }

  /// 保存工作流到本地文件
  pub async fn save_workflow_to_file(&self, workflow: &Workflow, path: &PathBuf) -> CliResult<()> {
    let content = serde_json::to_string_pretty(workflow)?;

    // 确保目录存在
    if let Some(parent) = path.parent() {
      tokio::fs::create_dir_all(parent).await.map_err(|e| CliError::io_error(parent.to_path_buf(), e))?;
    }

    tokio::fs::write(path, content).await.map_err(|e| CliError::io_error(path.clone(), e))?;

    Ok(())
  }

  /// 从本地文件加载执行请求数据
  pub async fn load_input_data_from_file(&self, path: &PathBuf) -> CliResult<ParameterMap> {
    let content = tokio::fs::read_to_string(path).await.map_err(|e| CliError::io_error(path.clone(), e))?;

    serde_json::from_str(&content).map_err(|e| CliError::validation_error(format!("解析输入数据文件失败: {e}")))
  }
}

#[cfg(test)]
mod tests {
  use ultimate_common::ahash::HashMap;

  use crate::config::{ApiConfig, CliConfig};

  use super::*;

  fn create_test_config() -> CliConfig {
    CliConfig { api: ApiConfig { endpoint: "http://localhost:8080".to_string(), token: "test-token".to_string() } }
  }

  #[test]
  fn test_api_client_creation() {
    let config = create_test_config();
    let client = ApiClient::new(config);
    assert!(client.is_ok());
  }

  #[tokio::test]
  async fn test_load_workflow_from_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let config = create_test_config();
    let client = ApiClient::new(config).unwrap();

    // 创建临时文件
    let mut temp_file = NamedTempFile::new().unwrap();
    let workflow_json = r#"
        {
            "id": "12345678-1234-1234-1234-123456789012",
            "name": "Test Workflow",
            "status": 1,
            "version_id": "12345678-1234-1234-1234-123456789012",
            "settings": {
                "execution_timeout": null,
                "max_concurrent_executions": null,
                "error_handling": 1,
                "execution_mode": 1,
                "save_execution_data_days": null,
                "remark": null
            },
            "meta": {
                "credentials_setup_completed": false
            },
            "nodes": [],
            "connections": [],
            "pin_data": {
                "data": {}
            },
            "static_data": null
        }
        "#;

    temp_file.write_all(workflow_json.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let result = client.load_workflow_from_file(&temp_path).await;
    assert!(result.is_ok());

    let workflow = result.unwrap();
    assert_eq!(workflow.name, "Test Workflow");
  }

  #[tokio::test]
  async fn test_load_workflow_from_invalid_file() {
    let config = create_test_config();
    let client = ApiClient::new(config).unwrap();

    use std::path::PathBuf;
    let invalid_path = PathBuf::from("/non/existent/file.json");

    let result = client.load_workflow_from_file(&invalid_path).await;
    assert!(result.is_err());

    if let Err(CliError::IoError { path, .. }) = result {
      assert_eq!(path, invalid_path);
    } else {
      panic!("Expected IoError");
    }
  }

  #[tokio::test]
  async fn test_load_workflow_from_invalid_json() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let config = create_test_config();
    let client = ApiClient::new(config).unwrap();

    // 创建包含无效 JSON 的临时文件
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"invalid json content").unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let result = client.load_workflow_from_file(&temp_path).await;
    assert!(result.is_err());

    if let Err(CliError::ValidationError { message }) = result {
      assert!(message.contains("解析工作流文件失败"));
    } else {
      panic!("Expected ValidationError");
    }
  }

  #[tokio::test]
  async fn test_save_workflow_to_file() {
    use hetumind_core::workflow::{
      ErrorHandlingStrategy, ExecutionMode, PinData, Workflow, WorkflowMeta, WorkflowSettings, WorkflowStatus,
    };
    use tempfile::tempdir;

    let config = create_test_config();
    let client = ApiClient::new(config).unwrap();

    // 创建测试工作流
    let workflow = Workflow {
      id: WorkflowId::now_v7(),
      name: "Test Save Workflow".to_string(),
      status: WorkflowStatus::Draft,
      version: None,
      settings: WorkflowSettings {
        execution_timeout: Some(300),
        error_handling: Some(ErrorHandlingStrategy::StopOnFirstError),
        execution_mode: Some(ExecutionMode::Local),
        remark: Some("测试工作流".to_string()),
      },
      meta: WorkflowMeta::default(),
      nodes: Vec::new(),
      connections: HashMap::default(),
      pin_data: PinData::default(),
      static_data: None,
    };

    // 创建临时目录
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test_workflow.json");

    // 保存工作流到文件
    let result = client.save_workflow_to_file(&workflow, &file_path).await;
    assert!(result.is_ok());

    // 验证文件是否创建
    assert!(file_path.exists());

    // 验证文件内容
    let content = std::fs::read_to_string(&file_path).unwrap();
    let saved_workflow: Workflow = serde_json::from_str(&content).unwrap();
    assert_eq!(saved_workflow.name, "Test Save Workflow");
    assert_eq!(saved_workflow.id, workflow.id);
  }

  #[tokio::test]
  async fn test_load_input_data_from_file() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let config = create_test_config();
    let client = ApiClient::new(config).unwrap();

    // 创建包含输入数据的临时文件
    let mut temp_file = NamedTempFile::new().unwrap();
    let input_json = r#"{"name": "test", "value": 42, "enabled": true}"#;
    temp_file.write_all(input_json.as_bytes()).unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let result = client.load_input_data_from_file(&temp_path).await;
    assert!(result.is_ok());

    let input_data = result.unwrap();
    assert_eq!(input_data.len(), 3);
    assert!(input_data.contains_key("name"));
    assert!(input_data.contains_key("value"));
    assert!(input_data.contains_key("enabled"));
  }

  #[tokio::test]
  async fn test_load_input_data_from_invalid_json() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let config = create_test_config();
    let client = ApiClient::new(config).unwrap();

    // 创建包含无效 JSON 的临时文件
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"{ invalid json }").unwrap();
    let temp_path = temp_file.path().to_path_buf();

    let result = client.load_input_data_from_file(&temp_path).await;
    assert!(result.is_err());

    if let Err(CliError::ValidationError { message }) = result {
      assert!(message.contains("解析输入数据文件失败"));
    } else {
      panic!("Expected ValidationError");
    }
  }

  #[test]
  fn test_api_client_url_generation() {
    let config = create_test_config();
    let client = ApiClient::new(config).unwrap();

    // 测试通过反射或公共方法访问内部配置
    // 由于 config 字段是私有的，我们通过构造函数和基本操作来验证配置正确性
    assert!(client.config.api.endpoint.contains("localhost"));
    assert_eq!(client.config.api.token, "test-token");
  }

  #[test]
  fn test_api_client_with_different_endpoints() {
    let mut config = create_test_config();
    config.api.endpoint = "https://api.example.com".to_string();

    let client = ApiClient::new(config);
    assert!(client.is_ok());

    let client = client.unwrap();
    assert!(client.config.api.endpoint.contains("example.com"));
  }

  #[test]
  fn test_api_client_with_empty_token() {
    let mut config = create_test_config();
    config.api.token = "".to_string();

    let client = ApiClient::new(config);
    assert!(client.is_ok());

    let client = client.unwrap();
    assert_eq!(client.config.api.token, "");
  }
}
