use super::{
  context::ExpressionExecutionContext,
  data_proxy::DataProxy,
  functions::{FunctionError, FunctionRegistry},
  parse::{BinaryOperator, Expression, ExpressionParser, ParseError},
  value::Value,
};
use ahash::HashMap;
use chrono::NaiveTime;
use fusion_common::time::{OffsetDateTime, now};

#[derive(Debug, thiserror::Error)]
pub enum EvaluationError {
  #[error("解析错误: {0}")]
  ParseError(#[from] ParseError),
  #[error("函数错误: {0}")]
  FunctionError(#[from] FunctionError),
  #[error("属性访问错误: 对象没有属性 {property}")]
  PropertyAccessError { property: String },
  #[error("索引访问错误: {message}")]
  IndexAccessError { message: String },
  #[error("方法调用错误: {message}")]
  MethodCallError { message: String },
  #[error("节点访问错误: 节点 {node_name} 不存在")]
  NodeAccessError { node_name: String },
  #[error("类型错误: {message}")]
  TypeError { message: String },
}

#[derive(Debug)]
pub struct ExpressionEvaluator {
  function_registry: FunctionRegistry,
}

impl Default for ExpressionEvaluator {
  fn default() -> Self {
    Self::new()
  }
}

impl ExpressionEvaluator {
  pub fn new() -> Self {
    Self { function_registry: FunctionRegistry::new() }
  }

  /// 评估表达式字符串
  pub fn evaluate(
    &self,
    expression: &str,
    data_proxy: &dyn DataProxy,
    context: &ExpressionExecutionContext,
  ) -> Result<Value, EvaluationError> {
    // 解析表达式
    let expr = ExpressionParser::parse(expression)?;

    // 评估表达式
    self.evaluate_expression(&expr, data_proxy, context)
  }

  /// 评估已解析的表达式
  fn evaluate_expression(
    &self,
    expr: &Expression,
    data_proxy: &dyn DataProxy,
    context: &ExpressionExecutionContext,
  ) -> Result<Value, EvaluationError> {
    match expr {
      Expression::Literal(value) => Ok(value.clone()),

      Expression::Variable(name) => self.evaluate_variable(name, data_proxy, context),

      Expression::PropertyAccess { object, property } => {
        let obj_value = self.evaluate_expression(object, data_proxy, context)?;
        self.evaluate_property_access(&obj_value, property)
      }

      Expression::IndexAccess { object, index } => {
        let obj_value = self.evaluate_expression(object, data_proxy, context)?;
        let index_value = self.evaluate_expression(index, data_proxy, context)?;
        self.evaluate_index_access(&obj_value, &index_value)
      }

      Expression::MethodCall { object, method, args } => {
        let obj_value = self.evaluate_expression(object, data_proxy, context)?;
        let arg_values: Result<Vec<_>, _> =
          args.iter().map(|arg| self.evaluate_expression(arg, data_proxy, context)).collect();
        self.evaluate_method_call(&obj_value, method, &arg_values?, data_proxy, context)
      }

      Expression::FunctionCall { name, args } => {
        let arg_values: Result<Vec<_>, _> =
          args.iter().map(|arg| self.evaluate_expression(arg, data_proxy, context)).collect();
        self.function_registry.call(name, &arg_values?, data_proxy, context).map_err(|e| e.into())
      }

      Expression::BinaryOp { left, operator, right } => {
        let left_value = self.evaluate_expression(left, data_proxy, context)?;
        let right_value = self.evaluate_expression(right, data_proxy, context)?;
        self.evaluate_binary_op(&left_value, operator, &right_value)
      }

      Expression::ConditionalExpr { condition, then_expr, else_expr } => {
        let cond_value = self.evaluate_expression(condition, data_proxy, context)?;
        if cond_value.is_truthy() {
          self.evaluate_expression(then_expr, data_proxy, context)
        } else {
          self.evaluate_expression(else_expr, data_proxy, context)
        }
      }

      Expression::JsonPath { path, data } => {
        let data_value = self.evaluate_expression(data, data_proxy, context)?;
        self
          .function_registry
          .call("$jsonpath", &[Value::String(path.clone()), data_value], data_proxy, context)
          .map_err(|e| e.into())
      }

      Expression::NodeAccess { node_name } => {
        if let Some(node_output) = data_proxy.get_node_output(node_name) {
          Ok(Value::Array(node_output.json.clone()))
        } else {
          Err(EvaluationError::NodeAccessError { node_name: node_name.clone() })
        }
      }

      Expression::InputAccess { method } => match method.as_deref() {
        Some("all") => Ok(Value::Array(data_proxy.get_input_all().into_iter().cloned().collect())),
        Some("first") => Ok(data_proxy.get_input_first().cloned().unwrap_or(Value::Null)),
        Some("last") => Ok(data_proxy.get_input_last().cloned().unwrap_or(Value::Null)),
        Some("item") => Ok(data_proxy.get_input_item().cloned().unwrap_or(Value::Null)),
        None => Ok(data_proxy.get_json().clone()),
        _ => Err(EvaluationError::MethodCallError {
          message: format!("未知的 $input 方法: {}", method.as_ref().unwrap()),
        }),
      },
    }
  }

  /// 评估变量
  fn evaluate_variable(
    &self,
    name: &str,
    data_proxy: &dyn DataProxy,
    context: &ExpressionExecutionContext,
  ) -> Result<Value, EvaluationError> {
    match name {
      "json" => Ok(data_proxy.get_json().clone()),
      "binary" => Ok(data_proxy.get_binary().cloned().unwrap_or(Value::Null)),
      "now" => Ok(Value::DateTime(now())),
      "today" => {
        let now = now();
        let today = now.with_time(NaiveTime::MIN).unwrap();
        Ok(Value::DateTime(today))
      }
      "workflow" => Ok(Value::Object(HashMap::from_iter([
        ("id".to_string(), Value::String(context.workflow.id.to_string())),
        ("name".to_string(), Value::String(context.workflow.name.clone())),
        ("active".to_string(), Value::Bool(context.workflow.status.is_active())),
      ]))),
      "execution" => Ok(Value::Object(HashMap::from_iter([
        ("id".to_string(), Value::String(context.execution.id.to_string())),
        ("mode".to_string(), Value::String(format!("{:?}", context.execution.mode))),
      ]))),
      "env" => Ok(Value::Object(context.env.iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect())),
      "vars" => Ok(Value::Object(context.vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect())),
      "http" => {
        // 返回 HTTP 相关信息
        let mut http_obj = HashMap::default();
        if let Some(pagination) = &context.http_pagination {
          let mut pagination_obj = HashMap::default();
          pagination_obj.insert("page".to_string(), Value::Number(pagination.page as f64));
          pagination_obj.insert("total".to_string(), Value::Number(pagination.total as f64));
          pagination_obj.insert("per_page".to_string(), Value::Number(pagination.per_page as f64));
          pagination_obj.insert("has_next".to_string(), Value::Bool(pagination.has_next));
          http_obj.insert("pagination".to_string(), Value::Object(pagination_obj));
        }
        Ok(Value::Object(http_obj))
      }
      "input" => {
        // 返回 input 对象
        let input_data: Vec<Value> = data_proxy.get_input_all().into_iter().cloned().collect();
        let mut input_obj = HashMap::default();

        // 添加输入数据本身
        input_obj.insert("all".to_string(), Value::Array(input_data.clone()));
        input_obj.insert("first".to_string(), data_proxy.get_input_first().cloned().unwrap_or(Value::Null));
        input_obj.insert("last".to_string(), data_proxy.get_input_last().cloned().unwrap_or(Value::Null));
        input_obj.insert("item".to_string(), data_proxy.get_input_item().cloned().unwrap_or(Value::Null));

        // items 方法
        for (i, item) in input_data.iter().enumerate() {
          input_obj.insert(format!("items[{i}]"), item.clone());
        }

        Ok(Value::Object(input_obj))
      }
      _ => {
        // 检查自定义变量
        if let Some(value) = context.get_var(name) {
          Ok(value.clone())
        } else {
          Err(EvaluationError::TypeError { message: format!("未知变量: '{}'", name) })
        }
      }
    }
  }

  /// 评估属性访问
  fn evaluate_property_access(&self, object: &Value, property: &str) -> Result<Value, EvaluationError> {
    match object {
      Value::Object(map) => map
        .get(property)
        .cloned()
        .ok_or_else(|| EvaluationError::PropertyAccessError { property: property.to_string() }),
      Value::String(s) => match property {
        "length" => Ok(Value::Number(s.len() as f64)),
        _ => Err(EvaluationError::PropertyAccessError { property: property.to_string() }),
      },
      Value::Array(arr) => match property {
        "length" => Ok(Value::Number(arr.len() as f64)),
        _ => Err(EvaluationError::PropertyAccessError { property: property.to_string() }),
      },
      _ => Err(EvaluationError::PropertyAccessError { property: property.to_string() }),
    }
  }

  /// 评估索引访问
  fn evaluate_index_access(&self, object: &Value, index: &Value) -> Result<Value, EvaluationError> {
    match (object, index) {
      (Value::Array(arr), Value::Number(n)) => {
        let idx = *n as usize;
        arr
          .get(idx)
          .cloned()
          .ok_or_else(|| EvaluationError::IndexAccessError { message: format!("索引 {} 超出数组范围", idx) })
      }
      (Value::Object(map), Value::String(key)) => map
        .get(key)
        .cloned()
        .ok_or_else(|| EvaluationError::IndexAccessError { message: format!("对象没有键: {}", key) }),
      _ => Err(EvaluationError::IndexAccessError { message: "不支持的索引访问操作".to_string() }),
    }
  }

  /// 评估方法调用
  fn evaluate_method_call(
    &self,
    object: &Value,
    method: &str,
    args: &[Value],
    data_proxy: &dyn DataProxy,
    context: &ExpressionExecutionContext,
  ) -> Result<Value, EvaluationError> {
    match object {
      Value::String(s) => self.evaluate_string_method(s, method, args),
      Value::Array(arr) => self.evaluate_array_method(arr, method, args, data_proxy, context),
      Value::DateTime(dt) => self.evaluate_datetime_method(dt, method, args),
      Value::Number(n) => self.evaluate_number_method(*n, method, args),
      Value::Object(obj) => self.evaluate_object_method(obj, method, args, data_proxy, context),
      _ => Err(EvaluationError::MethodCallError { message: format!("类型 {:?} 不支持方法调用", object) }),
    }
  }

  /// 字符串方法
  fn evaluate_string_method(&self, s: &str, method: &str, args: &[Value]) -> Result<Value, EvaluationError> {
    match method {
      "toUpperCase" => Ok(Value::String(s.to_uppercase())),
      "toLowerCase" => Ok(Value::String(s.to_lowercase())),
      "trim" => Ok(Value::String(s.trim().to_string())),
      "length" => Ok(Value::Number(s.len() as f64)),
      "split" => {
        if let Some(Value::String(separator)) = args.first() {
          let parts: Vec<Value> = s.split(separator).map(|part| Value::String(part.to_string())).collect();
          Ok(Value::Array(parts))
        } else {
          Err(EvaluationError::MethodCallError { message: "split 方法需要一个字符串参数".to_string() })
        }
      }
      "replace" => {
        if args.len() >= 2 {
          if let (Value::String(from), Value::String(to)) = (&args[0], &args[1]) {
            Ok(Value::String(s.replace(from, to)))
          } else {
            Err(EvaluationError::MethodCallError { message: "replace 方法需要两个字符串参数".to_string() })
          }
        } else {
          Err(EvaluationError::MethodCallError { message: "replace 方法需要两个参数".to_string() })
        }
      }
      "slice" => {
        if !args.is_empty() {
          if let Value::Number(start) = &args[0] {
            let start_idx = (*start as usize).min(s.len());
            let end_idx = if args.len() >= 2 {
              if let Value::Number(end) = &args[1] { (*end as usize).min(s.len()) } else { s.len() }
            } else {
              s.len()
            };
            Ok(Value::String(s[start_idx..end_idx].to_string()))
          } else {
            Err(EvaluationError::MethodCallError { message: "slice 方法需要数字参数".to_string() })
          }
        } else {
          Err(EvaluationError::MethodCallError { message: "slice 方法需要至少一个参数".to_string() })
        }
      }
      "includes" => {
        if let Some(Value::String(needle)) = args.first() {
          Ok(Value::Bool(s.contains(needle)))
        } else {
          Err(EvaluationError::MethodCallError { message: "includes 方法需要一个字符串参数".to_string() })
        }
      }
      "startsWith" => {
        if let Some(Value::String(prefix)) = args.first() {
          Ok(Value::Bool(s.starts_with(prefix)))
        } else {
          Err(EvaluationError::MethodCallError { message: "startsWith 方法需要一个字符串参数".to_string() })
        }
      }
      "endsWith" => {
        if let Some(Value::String(suffix)) = args.first() {
          Ok(Value::Bool(s.ends_with(suffix)))
        } else {
          Err(EvaluationError::MethodCallError { message: "endsWith 方法需要一个字符串参数".to_string() })
        }
      }
      "extractEmail" => {
        use regex::Regex;
        let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        if let Some(mat) = email_regex.find(s) { Ok(Value::String(mat.as_str().to_string())) } else { Ok(Value::Null) }
      }
      "toTitleCase" => {
        let title_case = s
          .split_whitespace()
          .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
              None => String::new(),
              Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
          })
          .collect::<Vec<_>>()
          .join(" ");
        Ok(Value::String(title_case))
      }
      "replaceSpecialChars" => {
        use regex::Regex;
        let special_chars = Regex::new(r"[^a-zA-Z0-9\s]").unwrap();
        Ok(Value::String(special_chars.replace_all(s, "").to_string()))
      }
      _ => Err(EvaluationError::MethodCallError { message: format!("字符串不支持方法: {}", method) }),
    }
  }

  /// 数组方法
  fn evaluate_array_method(
    &self,
    arr: &[Value],
    method: &str,
    args: &[Value],
    _data_proxy: &dyn DataProxy,
    _context: &ExpressionExecutionContext,
  ) -> Result<Value, EvaluationError> {
    match method {
      "length" => Ok(Value::Number(arr.len() as f64)),
      "first" => Ok(arr.first().cloned().unwrap_or(Value::Null)),
      "last" => Ok(arr.last().cloned().unwrap_or(Value::Null)),
      "join" => {
        let separator = if let Some(Value::String(sep)) = args.first() { sep.clone() } else { ",".to_string() };
        let joined = arr
          .iter()
          .map(|v| match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => "".to_string(),
          })
          .collect::<Vec<_>>()
          .join(&separator);
        Ok(Value::String(joined))
      }
      "reverse" => {
        let mut reversed = arr.to_vec();
        reversed.reverse();
        Ok(Value::Array(reversed))
      }
      "sort" => {
        let mut sorted = arr.to_vec();
        sorted.sort_by(|a, b| match (a, b) {
          (Value::Number(n1), Value::Number(n2)) => n1.partial_cmp(n2).unwrap_or(std::cmp::Ordering::Equal),
          (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
          _ => std::cmp::Ordering::Equal,
        });
        Ok(Value::Array(sorted))
      }
      "filter" => {
        // 简化的 filter 实现，只支持简单条件
        if let Some(condition) = args.first() {
          match condition {
            Value::String(prop) => {
              // 过滤有指定属性的对象
              let filtered: Vec<Value> = arr
                .iter()
                .filter(|item| if let Value::Object(obj) = item { obj.contains_key(prop) } else { false })
                .cloned()
                .collect();
              Ok(Value::Array(filtered))
            }
            _ => Ok(Value::Array(arr.to_vec())),
          }
        } else {
          Ok(Value::Array(arr.to_vec()))
        }
      }
      "map" => {
        // 简化的 map 实现
        if let Some(Value::String(prop)) = args.first() {
          let mapped: Vec<Value> = arr
            .iter()
            .map(|item| {
              if let Value::Object(obj) = item { obj.get(prop).cloned().unwrap_or(Value::Null) } else { Value::Null }
            })
            .collect();
          Ok(Value::Array(mapped))
        } else {
          Ok(Value::Array(arr.to_vec()))
        }
      }
      _ => Err(EvaluationError::MethodCallError { message: format!("数组不支持方法: {}", method) }),
    }
  }

  /// 时间方法
  fn evaluate_datetime_method(
    &self,
    dt: &OffsetDateTime,
    method: &str,
    args: &[Value],
  ) -> Result<Value, EvaluationError> {
    match method {
      "toFormat" => {
        if let Some(Value::String(format)) = args.first() {
          Ok(Value::String(dt.format(format).to_string()))
        } else {
          Err(EvaluationError::MethodCallError { message: "toFormat 方法需要格式字符串参数".to_string() })
        }
      }
      "plus" => {
        if let Some(Value::Object(duration_obj)) = args.first() {
          let mut new_dt = *dt;

          if let Some(Value::Number(days)) = duration_obj.get("days") {
            new_dt += chrono::Duration::days(*days as i64);
          }
          if let Some(Value::Number(hours)) = duration_obj.get("hours") {
            new_dt += chrono::Duration::hours(*hours as i64);
          }
          if let Some(Value::Number(minutes)) = duration_obj.get("minutes") {
            new_dt += chrono::Duration::minutes(*minutes as i64);
          }
          if let Some(Value::Number(seconds)) = duration_obj.get("seconds") {
            new_dt += chrono::Duration::seconds(*seconds as i64);
          }

          Ok(Value::DateTime(new_dt))
        } else {
          Err(EvaluationError::MethodCallError { message: "plus 方法需要持续时间对象参数".to_string() })
        }
      }
      "minus" => {
        if let Some(Value::Object(duration_obj)) = args.first() {
          let mut new_dt = *dt;

          if let Some(Value::Number(days)) = duration_obj.get("days") {
            new_dt -= chrono::Duration::days(*days as i64);
          }
          if let Some(Value::Number(hours)) = duration_obj.get("hours") {
            new_dt -= chrono::Duration::hours(*hours as i64);
          }
          if let Some(Value::Number(minutes)) = duration_obj.get("minutes") {
            new_dt -= chrono::Duration::minutes(*minutes as i64);
          }
          if let Some(Value::Number(seconds)) = duration_obj.get("seconds") {
            new_dt -= chrono::Duration::seconds(*seconds as i64);
          }

          Ok(Value::DateTime(new_dt))
        } else {
          Err(EvaluationError::MethodCallError { message: "minus 方法需要持续时间对象参数".to_string() })
        }
      }
      _ => Err(EvaluationError::MethodCallError { message: format!("DateTime 不支持方法: {}", method) }),
    }
  }

  /// 数字方法
  fn evaluate_number_method(&self, n: f64, method: &str, _args: &[Value]) -> Result<Value, EvaluationError> {
    match method {
      "abs" => Ok(Value::Number(n.abs())),
      "ceil" => Ok(Value::Number(n.ceil())),
      "floor" => Ok(Value::Number(n.floor())),
      "round" => Ok(Value::Number(n.round())),
      "toString" => Ok(Value::String(n.to_string())),
      _ => Err(EvaluationError::MethodCallError { message: format!("数字不支持方法: {}", method) }),
    }
  }

  /// 对象方法
  fn evaluate_object_method(
    &self,
    obj: &HashMap<String, Value>,
    method: &str,
    _args: &[Value],
    _data_proxy: &dyn DataProxy,
    _context: &ExpressionExecutionContext,
  ) -> Result<Value, EvaluationError> {
    match method {
      "all" => {
        if let Some(Value::Array(arr)) = obj.get("all") {
          Ok(Value::Array(arr.clone()))
        } else {
          Err(EvaluationError::MethodCallError { message: "对象没有 all 数据".to_string() })
        }
      }
      "first" => {
        if let Some(value) = obj.get("first") {
          Ok(value.clone())
        } else {
          Ok(Value::Null)
        }
      }
      "last" => {
        if let Some(value) = obj.get("last") {
          Ok(value.clone())
        } else {
          Ok(Value::Null)
        }
      }
      "item" => {
        if let Some(value) = obj.get("item") {
          Ok(value.clone())
        } else {
          Ok(Value::Null)
        }
      }
      _ => Err(EvaluationError::MethodCallError { message: format!("对象不支持方法: {}", method) }),
    }
  }

  /// 评估二元运算符
  fn evaluate_binary_op(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Result<Value, EvaluationError> {
    match op {
      BinaryOperator::Add => match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::String(a), Value::Number(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::Number(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
        _ => Err(EvaluationError::TypeError { message: format!("不能执行 {:?} + {:?}", left, right) }),
      },
      BinaryOperator::Sub => match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
        _ => Err(EvaluationError::TypeError { message: format!("不能执行 {:?} - {:?}", left, right) }),
      },
      BinaryOperator::Mul => match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
        _ => Err(EvaluationError::TypeError { message: format!("不能执行 {:?} * {:?}", left, right) }),
      },
      BinaryOperator::Div => match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
          if *b == 0.0 {
            Err(EvaluationError::TypeError { message: "除零错误".to_string() })
          } else {
            Ok(Value::Number(a / b))
          }
        }
        _ => Err(EvaluationError::TypeError { message: format!("不能执行 {:?} / {:?}", left, right) }),
      },
      BinaryOperator::Mod => match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
          if *b == 0.0 {
            Err(EvaluationError::TypeError { message: "除零错误".to_string() })
          } else {
            Ok(Value::Number(a % b))
          }
        }
        _ => Err(EvaluationError::TypeError { message: format!("不能执行 {:?} % {:?}", left, right) }),
      },
      BinaryOperator::Equal => Ok(Value::Bool(left == right)),
      BinaryOperator::NotEqual => Ok(Value::Bool(left != right)),
      BinaryOperator::Less => match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
        _ => Ok(Value::Bool(false)),
      },
      BinaryOperator::Greater => match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
        _ => Ok(Value::Bool(false)),
      },
      BinaryOperator::LessEqual => match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
        _ => Ok(Value::Bool(false)),
      },
      BinaryOperator::GreaterEqual => match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
        _ => Ok(Value::Bool(false)),
      },
      BinaryOperator::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
      BinaryOperator::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),
    }
  }
}
