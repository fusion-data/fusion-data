/// `cargo test -p hetumind-core expression::tests -- --nocapture`
use context::ExpressionExecutionContext;
use data_proxy::{DefaultDataProxy, NodeOutput};
use evaluator::ExpressionEvaluator;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use value::Value;

use crate::workflow::{Execution, ExecutionId, Workflow, WorkflowId};

use super::*;

#[test]
fn test_evaluate_simple_expressions() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  // 数字
  let result = evaluator.evaluate("42", &proxy, &context).unwrap();
  assert_eq!(result, Value::Number(42.0));

  // 字符串
  let result = evaluator.evaluate("\"hello\"", &proxy, &context).unwrap();
  assert_eq!(result, Value::String("hello".to_string()));

  // 布尔值
  let result = evaluator.evaluate("true", &proxy, &context).unwrap();
  assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_evaluate_binary_operations() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  // 加法
  let result = evaluator.evaluate("1 + 2", &proxy, &context).unwrap();
  assert_eq!(result, Value::Number(3.0));

  // 字符串连接
  let result = evaluator.evaluate("\"hello\" + \" world\"", &proxy, &context).unwrap();
  assert_eq!(result, Value::String("hello world".to_string()));

  // 比较
  let result = evaluator.evaluate("5 > 3", &proxy, &context).unwrap();
  assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_evaluate_conditional() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  let result = evaluator.evaluate("true ? \"yes\" : \"no\"", &proxy, &context).unwrap();
  assert_eq!(result, Value::String("yes".to_string()));

  let result = evaluator.evaluate("false ? \"yes\" : \"no\"", &proxy, &context).unwrap();
  assert_eq!(result, Value::String("no".to_string()));
}

#[test]
fn test_evaluate_variables() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  let result = evaluator.evaluate("$now", &proxy, &context).unwrap();
  assert!(matches!(result, Value::DateTime(_)));

  let result = evaluator.evaluate("$workflow.name", &proxy, &context).unwrap();
  assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_basic_literal_expressions() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  // 数字
  assert_eq!(evaluator.evaluate("42", &proxy, &context).unwrap(), Value::Number(42.0));
  assert_eq!(evaluator.evaluate("3.15", &proxy, &context).unwrap(), Value::Number(3.15));

  // 字符串
  assert_eq!(evaluator.evaluate("\"hello\"", &proxy, &context).unwrap(), Value::String("hello".to_string()));
  assert_eq!(evaluator.evaluate("'world'", &proxy, &context).unwrap(), Value::String("world".to_string()));

  // 布尔值
  assert_eq!(evaluator.evaluate("true", &proxy, &context).unwrap(), Value::Bool(true));
  assert_eq!(evaluator.evaluate("false", &proxy, &context).unwrap(), Value::Bool(false));

  // null
  assert_eq!(evaluator.evaluate("null", &proxy, &context).unwrap(), Value::Null);
}

#[test]
fn test_basic_variables() {
  let evaluator = ExpressionEvaluator::new();
  let json_data = Value::Object(HashMap::from_iter([
    ("name".to_string(), Value::String("John".to_string())),
    ("age".to_string(), Value::Number(30.0)),
    ("labels".to_string(), Value::Array(vec![Value::String("developer".to_string()), Value::String("rust".to_string())])),
  ]));
  let proxy = DefaultDataProxy::new(json_data);
  let context = create_context_test_data();

  // $json 访问
  let result = evaluator.evaluate("$json", &proxy, &context).unwrap();
  assert!(matches!(result, Value::Object(_)));

  // $now 当前时间
  let result = evaluator.evaluate("$now", &proxy, &context).unwrap();
  assert!(matches!(result, Value::DateTime(_)));

  // $workflow 访问
  let result = evaluator.evaluate("$workflow.name", &proxy, &context).unwrap();
  assert_eq!(result, Value::new_string("Test Workflow"));

  let result = evaluator.evaluate("$workflow.id", &proxy, &context).unwrap();
  assert_eq!(result, Value::new_string("0197c9ef-5c91-7162-bf06-6f8fff516a40"));
}

#[test]
fn test_property_access() {
  let evaluator = ExpressionEvaluator::new();
  let json_data = Value::Object(HashMap::from_iter([(
    "user".to_string(),
    Value::Object(HashMap::from_iter([
      ("name".to_string(), Value::String("Alice".to_string())),
      (
        "profile".to_string(),
        Value::Object(HashMap::from_iter([
          ("email".to_string(), Value::String("alice@example.com".to_string())),
          ("age".to_string(), Value::Number(25.0)),
        ])),
      ),
    ])),
  )]));
  let proxy = DefaultDataProxy::new(json_data);
  let context = create_context_test_data();

  // 简单属性访问
  let result = evaluator.evaluate("$json.user.name", &proxy, &context).unwrap();
  assert_eq!(result, Value::String("Alice".to_string()));

  // 嵌套属性访问
  let result = evaluator.evaluate("$json.user.profile.email", &proxy, &context).unwrap();
  assert_eq!(result, Value::String("alice@example.com".to_string()));

  let result = evaluator.evaluate("$json.user.profile.age", &proxy, &context).unwrap();
  assert_eq!(result, Value::Number(25.0));
}

#[test]
fn test_array_access() {
  let evaluator = ExpressionEvaluator::new();
  let json_data = Value::Object(HashMap::from_iter([
    (
      "labels".to_string(),
      Value::Array(vec![
        Value::String("rust".to_string()),
        Value::String("programming".to_string()),
        Value::String("backend".to_string()),
      ]),
    ),
    ("scores".to_string(), Value::Array(vec![Value::Number(95.0), Value::Number(87.0), Value::Number(92.0)])),
  ]));
  let proxy = DefaultDataProxy::new(json_data);
  let context = create_context_test_data();

  // 数组索引访问
  let result = evaluator.evaluate("$json.labels[0]", &proxy, &context).unwrap();
  assert_eq!(result, Value::String("rust".to_string()));

  let result = evaluator.evaluate("$json.labels[1]", &proxy, &context).unwrap();
  assert_eq!(result, Value::String("programming".to_string()));

  let result = evaluator.evaluate("$json.scores[0]", &proxy, &context).unwrap();
  assert_eq!(result, Value::Number(95.0));
}

#[test]
fn test_binary_operations() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  // 算术运算
  assert_eq!(evaluator.evaluate("1 + 2", &proxy, &context).unwrap(), Value::Number(3.0));
  assert_eq!(evaluator.evaluate("10 - 3", &proxy, &context).unwrap(), Value::Number(7.0));
  assert_eq!(evaluator.evaluate("4 * 5", &proxy, &context).unwrap(), Value::Number(20.0));
  assert_eq!(evaluator.evaluate("15 / 3", &proxy, &context).unwrap(), Value::Number(5.0));
  assert_eq!(evaluator.evaluate("17 % 5", &proxy, &context).unwrap(), Value::Number(2.0));

  // 字符串连接
  assert_eq!(
    evaluator.evaluate("\"hello\" + \" world\"", &proxy, &context).unwrap(),
    Value::String("hello world".to_string())
  );
  assert_eq!(evaluator.evaluate("\"value: \" + 42", &proxy, &context).unwrap(), Value::String("value: 42".to_string()));

  // 比较运算
  assert_eq!(evaluator.evaluate("5 > 3", &proxy, &context).unwrap(), Value::Bool(true));
  assert_eq!(evaluator.evaluate("2 < 1", &proxy, &context).unwrap(), Value::Bool(false));
  assert_eq!(evaluator.evaluate("5 >= 5", &proxy, &context).unwrap(), Value::Bool(true));
  assert_eq!(evaluator.evaluate("4 <= 3", &proxy, &context).unwrap(), Value::Bool(false));
  assert_eq!(evaluator.evaluate("\"abc\" == \"abc\"", &proxy, &context).unwrap(), Value::Bool(true));
  assert_eq!(evaluator.evaluate("\"abc\" != \"def\"", &proxy, &context).unwrap(), Value::Bool(true));

  // 逻辑运算
  assert_eq!(evaluator.evaluate("true && true", &proxy, &context).unwrap(), Value::Bool(true));
  assert_eq!(evaluator.evaluate("true && false", &proxy, &context).unwrap(), Value::Bool(false));
  assert_eq!(evaluator.evaluate("false || true", &proxy, &context).unwrap(), Value::Bool(true));
  assert_eq!(evaluator.evaluate("false || false", &proxy, &context).unwrap(), Value::Bool(false));
}

#[test]
fn test_conditional_expressions() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  // 简单条件表达式
  assert_eq!(
    evaluator.evaluate("true ? \"yes\" : \"no\"", &proxy, &context).unwrap(),
    Value::String("yes".to_string())
  );
  assert_eq!(
    evaluator.evaluate("false ? \"yes\" : \"no\"", &proxy, &context).unwrap(),
    Value::String("no".to_string())
  );

  // 复杂条件表达式
  assert_eq!(
    evaluator.evaluate("5 > 3 ? \"big\" : \"small\"", &proxy, &context).unwrap(),
    Value::String("big".to_string())
  );
  assert_eq!(
    evaluator.evaluate("2 > 5 ? \"big\" : \"small\"", &proxy, &context).unwrap(),
    Value::String("small".to_string())
  );

  // 嵌套条件表达式
  assert_eq!(
    evaluator
      .evaluate("true ? (1 > 0 ? \"nested_true\" : \"nested_false\") : \"outer_false\"", &proxy, &context)
      .unwrap(),
    Value::String("nested_true".to_string())
  );
}

#[test]
fn test_string_methods() {
  let evaluator = ExpressionEvaluator::new();
  let json_data = Value::Object(HashMap::from_iter([
    ("text".to_string(), Value::String("Hello World".to_string())),
    ("email_text".to_string(), Value::String("Contact us at support@example.com for help".to_string())),
  ]));
  let proxy = DefaultDataProxy::new(json_data);
  let context = create_context_test_data();

  // 字符串方法测试
  assert_eq!(
    evaluator.evaluate("$json.text.toUpperCase()", &proxy, &context).unwrap(),
    Value::String("HELLO WORLD".to_string())
  );

  assert_eq!(
    evaluator.evaluate("$json.text.toLowerCase()", &proxy, &context).unwrap(),
    Value::String("hello world".to_string())
  );

  assert_eq!(evaluator.evaluate("$json.text.length()", &proxy, &context).unwrap(), Value::Number(11.0));

  assert_eq!(
    evaluator.evaluate("$json.text.split(\" \")", &proxy, &context).unwrap(),
    Value::Array(vec![Value::String("Hello".to_string()), Value::String("World".to_string())])
  );

  assert_eq!(
    evaluator.evaluate("$json.text.replace(\"World\", \"Rust\")", &proxy, &context).unwrap(),
    Value::String("Hello Rust".to_string())
  );

  assert_eq!(evaluator.evaluate("$json.text.includes(\"World\")", &proxy, &context).unwrap(), Value::Bool(true));

  assert_eq!(evaluator.evaluate("$json.text.startsWith(\"Hello\")", &proxy, &context).unwrap(), Value::Bool(true));

  assert_eq!(evaluator.evaluate("$json.text.endsWith(\"World\")", &proxy, &context).unwrap(), Value::Bool(true));

  // 特殊字符串方法
  assert_eq!(
    evaluator.evaluate("$json.text.toTitleCase()", &proxy, &context).unwrap(),
    Value::String("Hello World".to_string())
  );

  // 邮箱提取
  assert_eq!(
    evaluator.evaluate("$json.email_text.extractEmail()", &proxy, &context).unwrap(),
    Value::String("support@example.com".to_string())
  );
}

#[test]
fn test_array_methods() {
  let evaluator = ExpressionEvaluator::new();
  let json_data = Value::Object(HashMap::from_iter([
    (
      "numbers".to_string(),
      Value::Array(vec![
        Value::Number(3.0),
        Value::Number(1.0),
        Value::Number(4.0),
        Value::Number(1.0),
        Value::Number(5.0),
      ]),
    ),
    (
      "users".to_string(),
      Value::Array(vec![
        Value::Object(HashMap::from_iter([
          ("name".to_string(), Value::String("Alice".to_string())),
          ("active".to_string(), Value::Bool(true)),
        ])),
        Value::Object(HashMap::from_iter([
          ("name".to_string(), Value::String("Bob".to_string())),
          ("active".to_string(), Value::Bool(false)),
        ])),
      ]),
    ),
  ]));
  let proxy = DefaultDataProxy::new(json_data);
  let context = create_context_test_data();

  // 数组基本方法
  assert_eq!(evaluator.evaluate("$json.numbers.length()", &proxy, &context).unwrap(), Value::Number(5.0));

  assert_eq!(evaluator.evaluate("$json.numbers.first()", &proxy, &context).unwrap(), Value::Number(3.0));

  assert_eq!(evaluator.evaluate("$json.numbers.last()", &proxy, &context).unwrap(), Value::Number(5.0));

  // 数组连接
  assert_eq!(
    evaluator.evaluate("$json.numbers.join(\",\")", &proxy, &context).unwrap(),
    Value::String("3,1,4,1,5".to_string())
  );

  // 数组排序
  let sorted_result = evaluator.evaluate("$json.numbers.sort()", &proxy, &context).unwrap();
  if let Value::Array(arr) = sorted_result {
    assert_eq!(
      arr,
      vec![Value::Number(1.0), Value::Number(1.0), Value::Number(3.0), Value::Number(4.0), Value::Number(5.0),]
    );
  } else {
    panic!("Expected array result");
  }

  // 数组映射 (简化版本，获取名称)
  let mapped_result = evaluator.evaluate("$json.users.map(\"name\")", &proxy, &context).unwrap();
  if let Value::Array(arr) = mapped_result {
    assert_eq!(arr, vec![Value::String("Alice".to_string()), Value::String("Bob".to_string()),]);
  } else {
    panic!("Expected array result");
  }
}

#[test]
fn test_datetime_methods() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  // 日期格式化
  let result = evaluator.evaluate("$now.toFormat(\"%Y-%m-%d\")", &proxy, &context);
  assert!(result.is_ok());
  if let Ok(Value::String(formatted)) = result {
    // 验证格式是否正确 (YYYY-MM-DD)
    assert!(formatted.len() == 10);
    assert!(formatted.contains('-'));
  }

  // 时间加法
  let result = evaluator.evaluate("$now.plus({\"days\": 7}).toFormat(\"%Y-%m-%d\")", &proxy, &context);
  assert!(result.is_ok());

  // 时间减法
  let result = evaluator.evaluate("$now.minus({\"hours\": 1}).toFormat(\"%H\")", &proxy, &context);
  assert!(result.is_ok());
}

#[test]
fn test_number_methods() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  // 数字方法
  assert_eq!(evaluator.evaluate("(-5).abs()", &proxy, &context).unwrap(), Value::Number(5.0));
  assert_eq!(evaluator.evaluate("(3.7).ceil()", &proxy, &context).unwrap(), Value::Number(4.0));
  assert_eq!(evaluator.evaluate("(3.7).floor()", &proxy, &context).unwrap(), Value::Number(3.0));
  assert_eq!(evaluator.evaluate("(3.7).round()", &proxy, &context).unwrap(), Value::Number(4.0));
  assert_eq!(evaluator.evaluate("(42).toString()", &proxy, &context).unwrap(), Value::String("42".to_string()));
}

#[test]
fn test_input_access() {
  let evaluator = ExpressionEvaluator::new();
  let input_data = Value::Array(vec![
    Value::Object(HashMap::from_iter([
      ("id".to_string(), Value::Number(1.0)),
      ("name".to_string(), Value::String("Item 1".to_string())),
    ])),
    Value::Object(HashMap::from_iter([
      ("id".to_string(), Value::Number(2.0)),
      ("name".to_string(), Value::String("Item 2".to_string())),
    ])),
  ]);
  let proxy = DefaultDataProxy::new(input_data);
  let context = create_context_test_data();

  // $input 访问方法
  let result = evaluator.evaluate("$input.all()", &proxy, &context).unwrap();
  if let Value::Array(arr) = result {
    assert_eq!(arr.len(), 2);
  } else {
    panic!("Expected array result");
  }

  let result = evaluator.evaluate("$input.first()", &proxy, &context).unwrap();
  if let Value::Object(obj) = result {
    assert_eq!(obj.get("id"), Some(&Value::Number(1.0)));
  } else {
    panic!("Expected object result");
  }

  let result = evaluator.evaluate("$input.last()", &proxy, &context).unwrap();
  if let Value::Object(obj) = result {
    assert_eq!(obj.get("id"), Some(&Value::Number(2.0)));
  } else {
    panic!("Expected object result");
  }
}

#[test]
fn test_complex_expressions() {
  let evaluator = ExpressionEvaluator::new();
  let json_data = Value::Object(HashMap::from_iter([
    (
      "users".to_string(),
      Value::Array(vec![
        Value::Object(HashMap::from_iter([
          ("name".to_string(), Value::String("Alice".to_string())),
          ("age".to_string(), Value::Number(25.0)),
          ("score".to_string(), Value::Number(95.0)),
        ])),
        Value::Object(HashMap::from_iter([
          ("name".to_string(), Value::String("Bob".to_string())),
          ("age".to_string(), Value::Number(30.0)),
          ("score".to_string(), Value::Number(87.0)),
        ])),
      ]),
    ),
    ("config".to_string(), Value::Object(HashMap::from_iter([("pass_score".to_string(), Value::Number(90.0))]))),
  ]));
  let proxy = DefaultDataProxy::new(json_data);
  let context = create_context_test_data();

  // 复杂表达式：比较和条件
  let result = evaluator
    .evaluate("$json.users[0].score > $json.config.pass_score ? \"pass\" : \"fail\"", &proxy, &context)
    .unwrap();
  assert_eq!(result, Value::String("pass".to_string()));

  // 复杂表达式：数学运算和属性访问
  let result = evaluator.evaluate("$json.users[0].age + $json.users[1].age", &proxy, &context).unwrap();
  assert_eq!(result, Value::Number(55.0));

  // 复杂表达式：字符串操作
  let result = evaluator
    .evaluate("\"User: \" + $json.users[0].name + \" (Age: \" + $json.users[0].age + \")\"", &proxy, &context)
    .unwrap();
  assert_eq!(result, Value::String("User: Alice (Age: 25)".to_string()));
}

#[test]
fn test_environment_and_context() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let mut context = create_context_test_data();

  // 添加环境变量
  context.env.insert("API_KEY".to_string(), "secret-key-123".to_string());
  context.env.insert("DEBUG".to_string(), "true".to_string());

  // 添加自定义变量
  context.set_var("custom_value", Value::Number(42.0));

  // 测试环境变量访问
  let result = evaluator.evaluate("$env[\"API_KEY\"]", &proxy, &context).unwrap();
  assert_eq!(result, Value::new_string("secret-key-123"));

  // 测试变量访问
  let result = evaluator.evaluate("$vars[\"custom_value\"]", &proxy, &context).unwrap();
  assert_eq!(result, Value::Number(42.0));

  // 测试工作流信息
  let result = evaluator.evaluate("$workflow.name", &proxy, &context).unwrap();
  assert_eq!(result, Value::new_string("Test Workflow"));

  let result = evaluator.evaluate("$workflow.id", &proxy, &context).unwrap();
  assert_eq!(result, Value::new_string("0197c9ef-5c91-7162-bf06-6f8fff516a40"));
}

#[test]
fn test_function_calls() {
  let evaluator = ExpressionEvaluator::new();
  let proxy = DefaultDataProxy::new(Value::Null);
  let context = create_context_test_data();

  // 测试一些内置函数
  let result = evaluator.evaluate("$max(1, 5, 3)", &proxy, &context);
  // 注意：这些函数需要在 FunctionRegistry 中实现
  match result {
    Ok(value) => println!("Function result: {:?}", value),
    Err(e) => println!("Function error (expected): {:?}", e),
  }

  let result = evaluator.evaluate("$min(1, 5, 3)", &proxy, &context);
  match result {
    Ok(value) => println!("Function result: {:?}", value),
    Err(e) => println!("Function error (expected): {:?}", e),
  }
}

#[test]
fn test_javascript_style_expressions() {
  let evaluator = ExpressionEvaluator::new();
  let json_data = Value::Object(HashMap::from_iter([
    ("text".to_string(), Value::String("hello world".to_string())),
    ("value".to_string(), Value::Number(42.0)),
  ]));
  let proxy = DefaultDataProxy::new(json_data);
  let context = create_context_test_data();

  // JavaScript 风格表达式（{{ ... }}）
  let result = evaluator.evaluate("{{$json.value * 2}}", &proxy, &context).unwrap();
  assert_eq!(result, Value::Number(84.0));

  let result = evaluator.evaluate("{{$json.text.split(\" \").length()}}", &proxy, &context).unwrap();
  assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_node_access() {
  let evaluator = ExpressionEvaluator::new();

  // 创建节点输出数据
  let node_output = NodeOutput {
    json: vec![Value::Object(HashMap::from_iter([
      ("result".to_string(), Value::String("success".to_string())),
      ("count".to_string(), Value::Number(5.0)),
    ]))],
    binary: None,
  };

  let mut node_outputs = HashMap::default();
  node_outputs.insert("previous_node".to_string(), node_output);

  let proxy = DefaultDataProxy::new(Value::Null).with_node_outputs(node_outputs);
  let context = create_context_test_data();

  // 节点访问
  let result = evaluator.evaluate("$(\"previous_node\")", &proxy, &context).unwrap();
  if let Value::Array(arr) = result {
    assert_eq!(arr.len(), 1);
    if let Value::Object(obj) = &arr[0] {
      assert_eq!(obj.get("result"), Some(&Value::String("success".to_string())));
    }
  } else {
    panic!("Expected array result");
  }
}

fn create_context_test_data() -> ExpressionExecutionContext {
  let workflow = Arc::new(create_workflow_test_data());
  let execution = Arc::new(create_execution_test_data(workflow.id.clone()));
  ExpressionExecutionContext::builder()
    .workflow(workflow)
    .execution(execution)
    .env(std::env::vars().collect())
    .build()
}

fn create_workflow_test_data() -> Workflow {
  Workflow::builder()
    .id(WorkflowId::from_str("0197c9ef-5c91-7162-bf06-6f8fff516a40").unwrap())
    .name("Test Workflow")
    .build()
}

fn create_execution_test_data(workflow_id: WorkflowId) -> Execution {
  Execution::builder().id(ExecutionId::now_v7()).workflow_id(workflow_id).build()
}
