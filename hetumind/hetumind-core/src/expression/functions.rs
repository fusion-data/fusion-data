// src/functions.rs
use ahash::HashMap;
use chrono::{Duration, NaiveTime};
use jsonpath_rust::JsonPath;
use log::error;
use regex::Regex;
use ultimate_common::time::now;

use super::{
  context::ExpressionExecutionContext,
  data_proxy::DataProxy,
  value::{DateTimeExt, Value},
};

pub type FunctionResult = Result<Value, FunctionError>;
pub type FunctionHandler =
  Box<dyn Fn(&[Value], &dyn DataProxy, &ExpressionExecutionContext) -> FunctionResult + Send + Sync + 'static>;

#[derive(Debug, thiserror::Error)]
pub enum FunctionError {
  #[error("参数错误: {message}")]
  ArgumentError { message: String },
  #[error("类型错误: 期望 {expected}, 得到 {actual}")]
  TypeError { expected: String, actual: String },
  #[error("运行时错误: {message}")]
  RuntimeError { message: String },
}

pub struct FunctionRegistry {
  functions: HashMap<String, FunctionHandler>,
}
impl std::fmt::Debug for FunctionRegistry {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let keys = self.functions.keys().take(10).collect::<Vec<_>>();
    write!(f, "FunctionRegistry {{ functions: {:?} }}", keys)
  }
}

impl Default for FunctionRegistry {
  fn default() -> Self {
    Self::new()
  }
}

impl FunctionRegistry {
  pub fn new() -> Self {
    let mut registry = Self { functions: HashMap::default() };
    registry.register_builtin_functions();
    registry
  }

  pub fn register<F>(&mut self, name: &str, handler: F)
  where
    F: Fn(&[Value], &dyn DataProxy, &ExpressionExecutionContext) -> FunctionResult + Send + Sync + 'static,
  {
    self.functions.insert(name.to_string(), Box::new(handler));
  }

  pub fn call(
    &self,
    name: &str,
    args: &[Value],
    proxy: &dyn DataProxy,
    context: &ExpressionExecutionContext,
  ) -> FunctionResult {
    if let Some(handler) = self.functions.get(name) {
      handler(args, proxy, context)
    } else {
      Err(FunctionError::RuntimeError { message: format!("未知函数: {}", name) })
    }
  }

  fn register_builtin_functions(&mut self) {
    // 时间处理函数
    self.register("$now", |_args, _proxy, _ctx| Ok(Value::DateTime(now())));

    self.register("$today", |_args, _proxy, _ctx| {
      let now = now();
      let today = now.with_time(NaiveTime::MIN).unwrap();
      Ok(Value::DateTime(today))
    });

    // 时间操作扩展
    self.register("plus", |args, _proxy, _ctx| {
      if args.len() != 2 {
        return Err(FunctionError::ArgumentError {
          message: "plus() 需要2个参数: (datetime, duration)".to_string()
        });
      }

      let datetime = match &args[0] {
        Value::DateTime(dt) => dt,
        _ => {
          return Err(FunctionError::TypeError { expected: "DateTime".to_string(), actual: format!("{:?}", args[0]) });
        }
      };

      let duration = match &args[1] {
        Value::Object(obj) => {
          let mut dur = Duration::zero();
          if let Some(Value::Number(days)) = obj.get("days") {
            dur += Duration::days(*days as i64);
          }
          if let Some(Value::Number(hours)) = obj.get("hours") {
            dur += Duration::hours(*hours as i64);
          }
          if let Some(Value::Number(minutes)) = obj.get("minutes") {
            dur += Duration::minutes(*minutes as i64);
          }
          dur
        }
        _ => {
          return Err(FunctionError::TypeError {
            expected: "Duration object".to_string(),
            actual: format!("{:?}", args[1]),
          });
        }
      };

      Ok(Value::DateTime(datetime.plus(duration)))
    });

    // JSONPath 查询函数（使用 jsonpath-rust）
    self.register("$jsonpath", |args, _proxy, _ctx| {
      if args.len() != 2 {
        return Err(FunctionError::ArgumentError { message: "$jsonpath() 需要2个参数: (path, data)".to_string() });
      }

      let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(FunctionError::TypeError { expected: "String".to_string(), actual: format!("{:?}", args[0]) }),
      };

      // 转换为 serde_json::Value
      let json_value = args[1].to_json_value();

      // 使用 jsonpath-rust 实现完整的 JSONPath 支持
      let results = json_value.query(path).map_err(|e| FunctionError::RuntimeError { message: e.to_string() })?;

      if results.is_empty() {
        return Ok(Value::Null);
      }

      let result = results[0]; // TODO: 只返回第一个结果

      // 将 serde_json::Value 转换回我们的 Value 类型
      let converted_result = match result {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
          if let Some(i) = n.as_i64() {
            Value::Number(i as f64)
          } else if let Some(f) = n.as_f64() {
            Value::Number(f)
          } else {
            Value::Null
          }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
          let converted_arr: Vec<Value> = arr
            .iter()
            .map(|v| Value::from_json_value(v.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| FunctionError::RuntimeError { message: e.to_string() })?;
          Value::Array(converted_arr)
        }
        serde_json::Value::Object(obj) => {
          let converted_obj: HashMap<String, Value> = obj
            .iter()
            .map(|(k, v)| Value::from_json_value(v.clone()).map(|value| (k.clone(), value)))
            .collect::<Result<_, _>>()
            .map_err(|e| FunctionError::RuntimeError { message: e.to_string() })?;
          Value::Object(converted_obj)
        }
      };
      Ok(converted_result)
    });

    // 条件函数
    self.register("$if", |args, _proxy, _ctx| {
      if args.len() != 3 {
        return Err(FunctionError::ArgumentError {
          message: "$if() 需要3个参数: (condition, then_value, else_value)".to_string(),
        });
      }
      if args[0].is_truthy() { Ok(args[1].clone()) } else { Ok(args[2].clone()) }
    });

    self.register("$ifEmpty", |args, _proxy, _ctx| {
      if args.len() != 2 {
        return Err(FunctionError::ArgumentError {
          message: "$ifEmpty() 需要2个参数: (value, defaultValue)".to_string(),
        });
      }

      let is_empty = match &args[0] {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Array(arr) => arr.is_empty(),
        Value::Object(obj) => obj.is_empty(),
        _ => false,
      };

      if is_empty { Ok(args[1].clone()) } else { Ok(args[0].clone()) }
    });

    // 数学函数
    self.register("$max", |args, _proxy, _ctx| {
      if args.is_empty() {
        return Err(FunctionError::ArgumentError { message: "$max() 至少需要一个参数".to_string() });
      }

      let mut max_val = f64::NEG_INFINITY;
      for arg in args {
        if let Value::Number(n) = arg {
          max_val = max_val.max(*n);
        } else {
          return Err(FunctionError::TypeError { expected: "Number".to_string(), actual: format!("{:?}", arg) });
        }
      }
      Ok(Value::Number(max_val))
    });

    self.register("$min", |args, _proxy, _ctx| {
      if args.is_empty() {
        return Err(FunctionError::ArgumentError { message: "$min() 至少需要一个参数".to_string() });
      }

      let mut min_val = f64::INFINITY;
      for arg in args {
        if let Value::Number(n) = arg {
          min_val = min_val.min(*n);
        } else {
          return Err(FunctionError::TypeError { expected: "Number".to_string(), actual: format!("{:?}", arg) });
        }
      }
      Ok(Value::Number(min_val))
    });

    // 字符串处理函数
    self.register("replaceSpecialChars", |args, _proxy, _ctx| {
      if args.is_empty() || args.len() > 2 {
        return Err(FunctionError::ArgumentError {
          message: "replaceSpecialChars() 需要1-2个参数: (text, [replacement])".to_string(),
        });
      }

      let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err(FunctionError::TypeError { expected: "String".to_string(), actual: format!("{:?}", args[0]) }),
      };

      let replacement = if args.len() > 1 {
        match &args[1] {
          Value::String(s) => s.as_str(),
          _ => "_",
        }
      } else {
        "_"
      };

      let special_chars = Regex::new(r"[^a-zA-Z0-9\s]").unwrap();
      let result = special_chars.replace_all(text, replacement);

      Ok(Value::String(result.to_string()))
    });

    self.register("toTitleCase", |args, _proxy, _ctx| {
      if args.len() != 1 {
        return Err(FunctionError::ArgumentError { message: "toTitleCase() 需要1个参数: (text)".to_string() });
      }

      let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err(FunctionError::TypeError { expected: "String".to_string(), actual: format!("{:?}", args[0]) }),
      };

      let result = text
        .split_whitespace()
        .map(|word| {
          let mut chars = word.chars();
          match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
          }
        })
        .collect::<Vec<_>>()
        .join(" ");

      Ok(Value::String(result))
    });

    self.register("extractEmail", |args, _proxy, _ctx| {
      if args.len() != 1 {
        return Err(FunctionError::ArgumentError { message: "extractEmail() 需要1个参数: (text)".to_string() });
      }

      let text = match &args[0] {
        Value::String(s) => s,
        _ => return Err(FunctionError::TypeError { expected: "String".to_string(), actual: format!("{:?}", args[0]) }),
      };

      let email_regex = Regex::new(r"[\w\.-]+@[\w\.-]+\.\w+").unwrap();
      if let Some(email_match) = email_regex.find(text) {
        Ok(Value::String(email_match.as_str().to_string()))
      } else {
        Ok(Value::Null)
      }
    });

    // 数组操作函数
    self.register("filter", |args, proxy, ctx| {
      if args.len() != 2 {
        return Err(FunctionError::ArgumentError {
          message: "filter() 需要2个参数: (array, condition)".to_string()
        });
      }

      let _array = match &args[0] {
        Value::Array(arr) => arr,
        _ => return Err(FunctionError::TypeError { expected: "Array".to_string(), actual: format!("{:?}", args[0]) }),
      };

      // 暂时简化 filter 实现
      // TODO: 实现完整的数组过滤支持
      Ok(args[0].clone())
    });
  }
}
