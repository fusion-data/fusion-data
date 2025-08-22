use ahash::HashMap;

use super::Value;

/// 数据代理，用于访问节点输入输出数据
pub trait DataProxy: Send + Sync {
  /// 获取当前节点的输入数据（JSON格式）
  fn get_json(&self) -> &Value;

  /// 获取当前节点的二进制数据
  fn get_binary(&self) -> Option<&Value>;

  /// 获取指定节点的输出数据
  fn get_node_output(&self, node_name: &str) -> Option<&NodeOutput>;

  /// 获取当前输入的所有项
  fn get_input_all(&self) -> Vec<&Value>;

  /// 获取当前输入的第一项
  fn get_input_first(&self) -> Option<&Value>;

  /// 获取当前输入的最后一项
  fn get_input_last(&self) -> Option<&Value>;

  /// 获取当前输入项
  fn get_input_item(&self) -> Option<&Value>;
}

/// 节点输出数据
#[derive(Debug, Clone)]
pub struct NodeOutput {
  pub json: Vec<Value>,
  pub binary: Option<Vec<Value>>,
}

impl NodeOutput {
  pub fn first(&self) -> Option<&Value> {
    self.json.first()
  }

  pub fn last(&self) -> Option<&Value> {
    self.json.last()
  }

  pub fn all(&self) -> &Vec<Value> {
    &self.json
  }

  pub fn json(&self) -> &Vec<Value> {
    &self.json
  }

  pub fn binary(&self) -> Option<&Vec<Value>> {
    self.binary.as_ref()
  }
}

/// 默认的数据代理实现
pub struct DefaultDataProxy {
  current_json: Value,
  current_binary: Option<Value>,
  node_outputs: HashMap<String, NodeOutput>,
  input_items: Vec<Value>,
  current_item_index: usize,
}

impl DefaultDataProxy {
  pub fn new(json: Value) -> Self {
    let input_items = match &json {
      Value::Array(items) => items.clone(),
      _ => vec![json.clone()],
    };

    Self {
      current_json: json,
      current_binary: None,
      node_outputs: HashMap::default(),
      input_items,
      current_item_index: 0,
    }
  }

  pub fn with_binary(mut self, binary: Value) -> Self {
    self.current_binary = Some(binary);
    self
  }

  pub fn with_node_outputs(mut self, outputs: HashMap<String, NodeOutput>) -> Self {
    self.node_outputs = outputs;
    self
  }

  pub fn set_current_item_index(&mut self, index: usize) {
    self.current_item_index = index;
  }
}

impl DataProxy for DefaultDataProxy {
  fn get_json(&self) -> &Value {
    &self.current_json
  }

  fn get_binary(&self) -> Option<&Value> {
    self.current_binary.as_ref()
  }

  fn get_node_output(&self, node_name: &str) -> Option<&NodeOutput> {
    self.node_outputs.get(node_name)
  }

  fn get_input_all(&self) -> Vec<&Value> {
    self.input_items.iter().collect()
  }

  fn get_input_first(&self) -> Option<&Value> {
    self.input_items.first()
  }

  fn get_input_last(&self) -> Option<&Value> {
    self.input_items.last()
  }

  fn get_input_item(&self) -> Option<&Value> {
    self.input_items.get(self.current_item_index)
  }
}
