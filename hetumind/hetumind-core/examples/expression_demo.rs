use std::sync::Arc;

use ahash::HashMap;
use hetumind_core::expression::*;
use hetumind_core::workflow::{Execution, ExecutionId, Workflow, WorkflowId};

/// `cargo run -p hetumind-core --example expression_demo`
fn main() {
  println!("=== Hetumind 表达式系统演示 ===\n");

  // 创建评估器
  let evaluator = ExpressionEvaluator::new();

  // 创建测试数据
  let json_data = create_sample_data();
  let proxy = DefaultDataProxy::new(json_data).with_node_outputs(create_node_outputs());

  // 创建执行上下文
  let context = create_execution_context();

  println!("📊 1. 基本数据访问");
  demo_basic_access(&evaluator, &proxy, &context);

  println!("\n⏰ 2. 时间处理");
  demo_time_operations(&evaluator, &proxy, &context);

  println!("\n🔍 3. 字符串处理");
  demo_string_operations(&evaluator, &proxy, &context);

  println!("\n🔢 4. 数学运算");
  demo_math_operations(&evaluator, &proxy, &context);

  println!("\n❓ 5. 条件表达式");
  demo_conditional_expressions(&evaluator, &proxy, &context);

  println!("\n📋 6. 数组操作");
  demo_array_operations(&evaluator, &proxy, &context);

  println!("\n🌐 7. 环境和上下文访问");
  demo_context_access(&evaluator, &proxy, &context);

  println!("\n🔗 8. 节点数据访问");
  demo_node_access(&evaluator, &proxy, &context);

  println!("\n🔧 9. 复杂表达式");
  demo_complex_expressions(&evaluator, &proxy, &context);

  println!("\n✨ 10. JavaScript 风格表达式");
  demo_javascript_expressions(&evaluator, &proxy, &context);

  println!("\n🎯 演示完成！");
}

fn create_sample_data() -> Value {
  Value::Object(HashMap::from_iter([
    (
      "user".to_string(),
      Value::Object(HashMap::from_iter([
        ("id".to_string(), Value::Number(123.0)),
        ("name".to_string(), Value::String("张三".to_string())),
        ("email".to_string(), Value::String("zhangsan@example.com".to_string())),
        ("age".to_string(), Value::Number(28.0)),
        ("active".to_string(), Value::Bool(true)),
        (
          "labels".to_string(),
          Value::Array(vec![
            Value::String("developer".to_string()),
            Value::String("rust".to_string()),
            Value::String("backend".to_string()),
          ]),
        ),
        (
          "profile".to_string(),
          Value::Object(HashMap::from_iter([
            ("city".to_string(), Value::String("北京".to_string())),
            ("company".to_string(), Value::String("Hetumind".to_string())),
            ("score".to_string(), Value::Number(95.5)),
          ])),
        ),
      ])),
    ),
    (
      "orders".to_string(),
      Value::Array(vec![
        Value::Object(HashMap::from_iter([
          ("id".to_string(), Value::Number(1001.0)),
          ("amount".to_string(), Value::Number(299.99)),
          ("status".to_string(), Value::String("completed".to_string())),
        ])),
        Value::Object(HashMap::from_iter([
          ("id".to_string(), Value::Number(1002.0)),
          ("amount".to_string(), Value::Number(599.99)),
          ("status".to_string(), Value::String("pending".to_string())),
        ])),
      ]),
    ),
    (
      "config".to_string(),
      Value::Object(HashMap::from_iter([
        ("max_score".to_string(), Value::Number(100.0)),
        ("min_score".to_string(), Value::Number(60.0)),
      ])),
    ),
  ]))
}

fn create_node_outputs() -> HashMap<String, NodeOutput> {
  let mut outputs = HashMap::default();

  outputs.insert(
    "database_query".to_string(),
    NodeOutput {
      json: vec![Value::Object(HashMap::from_iter([
        ("total_users".to_string(), Value::Number(1250.0)),
        ("active_users".to_string(), Value::Number(980.0)),
        ("last_updated".to_string(), Value::String("2024-01-15 10:30:00".to_string())),
      ]))],
      binary: None,
    },
  );

  outputs.insert(
    "api_call".to_string(),
    NodeOutput {
      json: vec![Value::Object(HashMap::from_iter([
        ("status".to_string(), Value::String("success".to_string())),
        ("response_time".to_string(), Value::Number(125.0)),
        ("data_size".to_string(), Value::Number(2048.0)),
      ]))],
      binary: None,
    },
  );

  outputs
}

fn create_execution_context() -> ExpressionExecutionContext {
  let workflow = Arc::new(Workflow::builder().id(WorkflowId::now_v7()).name("Test Workflow").build());
  let execution = Arc::new(Execution::builder().id(ExecutionId::now_v7()).workflow_id(workflow.id.clone()).build());
  let mut context = ExpressionExecutionContext::builder().workflow(workflow).execution(execution).build();

  // 添加环境变量
  context.env.insert("API_KEY".to_string(), "sk-1234567890abcdef".to_string());
  context.env.insert("DATABASE_URL".to_string(), "postgresql://localhost:5432/analytics".to_string());
  context.env.insert("DEBUG".to_string(), "true".to_string());

  // 添加自定义变量
  context.set_var("threshold".to_string(), Value::Number(80.0));
  context.set_var(
    "feature_flags".to_string(),
    Value::Object(HashMap::from_iter([
      ("enable_cache".to_string(), Value::Bool(true)),
      ("max_retries".to_string(), Value::Number(3.0)),
    ])),
  );

  // 设置 HTTP 分页信息
  context.set_http_pagination(HttpPagination { page: 2, total: 150, per_page: 25, has_next: true });

  context
}

fn demo_basic_access(evaluator: &ExpressionEvaluator, proxy: &DefaultDataProxy, context: &ExpressionExecutionContext) {
  let examples = vec![
    ("$json", "获取完整的 JSON 数据"),
    ("$json.user.name", "获取用户名称"),
    ("$json.user.profile.city", "获取用户城市"),
    ("$json.user.labels[0]", "获取第一个标签"),
    ("$json.orders[1].amount", "获取第二个订单金额"),
    ("$now", "获取当前时间"),
    ("$workflow.name", "获取工作流名称"),
  ];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {desc} -> {:?}", result),
      Err(e) => println!("  {desc} -> 错误: {:?}", e),
    }
  }
}

fn demo_time_operations(
  evaluator: &ExpressionEvaluator,
  proxy: &DefaultDataProxy,
  context: &ExpressionExecutionContext,
) {
  let examples = vec![
    ("$now.toFormat(\"%Y-%m-%d %H:%M:%S\")", "格式化当前时间"),
    ("$now.plus({\"days\": 7}).toFormat(\"%Y-%m-%d\")", "7天后的日期"),
    ("$now.minus({\"hours\": 2}).toFormat(\"%H:%M\")", "2小时前的时间"),
    ("$today", "今天的日期（零时零分）"),
  ];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {} -> {:?}", desc, result),
      Err(e) => println!("  {} -> 错误: {:?}", desc, e),
    }
  }
}

fn demo_string_operations(
  evaluator: &ExpressionEvaluator,
  proxy: &DefaultDataProxy,
  context: &ExpressionExecutionContext,
) {
  let examples = vec![
    ("$json.user.name.toUpperCase()", "转换为大写"),
    ("$json.user.email.toLowerCase()", "转换为小写"),
    ("$json.user.email.split(\"@\")", "按 @ 分割邮箱"),
    ("$json.user.name.length()", "获取字符串长度"),
    ("$json.user.email.includes(\"example\")", "检查是否包含 example"),
    ("$json.user.email.startsWith(\"zhang\")", "检查是否以 zhang 开头"),
    ("$json.user.email.extractEmail()", "提取邮箱地址"),
    ("\"Hello World\".toTitleCase()", "转换为标题格式"),
    ("\"Contact me at support@test.com\".extractEmail()", "从文本中提取邮箱"),
  ];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {} -> {:?}", desc, result),
      Err(e) => println!("  {} -> 错误: {:?}", desc, e),
    }
  }
}

fn demo_math_operations(
  evaluator: &ExpressionEvaluator,
  proxy: &DefaultDataProxy,
  context: &ExpressionExecutionContext,
) {
  let examples = vec![
    ("$json.user.age + 10", "年龄加10"),
    ("$json.orders[0].amount * 1.1", "第一个订单金额乘以1.1"),
    ("$json.config.max_score - $json.config.min_score", "最大分数减最小分数"),
    ("$json.user.profile.score / 10", "分数除以10"),
    ("$json.user.age % 10", "年龄的个位数"),
    ("(-5.7).abs()", "绝对值"),
    ("(3.7).ceil()", "向上取整"),
    ("(3.7).floor()", "向下取整"),
    ("(3.5).round()", "四舍五入"),
  ];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {} -> {:?}", desc, result),
      Err(e) => println!("  {} -> 错误: {:?}", desc, e),
    }
  }
}

fn demo_conditional_expressions(
  evaluator: &ExpressionEvaluator,
  proxy: &DefaultDataProxy,
  context: &ExpressionExecutionContext,
) {
  let examples = vec![
    ("$json.user.age >= 18 ? \"成年\" : \"未成年\"", "年龄判断"),
    ("$json.user.active ? \"活跃用户\" : \"非活跃用户\"", "用户状态判断"),
    ("$json.user.profile.score > 90 ? \"优秀\" : \"良好\"", "分数等级判断"),
    ("$json.orders[0].status == \"completed\" ? \"已完成\" : \"进行中\"", "订单状态判断"),
    ("$json.user.age > 25 && $json.user.active", "复合条件判断"),
    ("$json.user.profile.score >= 95 || $json.user.labels.length() > 2", "或条件判断"),
  ];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {} -> {:?}", desc, result),
      Err(e) => println!("  {} -> 错误: {:?}", desc, e),
    }
  }
}

fn demo_array_operations(
  evaluator: &ExpressionEvaluator,
  proxy: &DefaultDataProxy,
  context: &ExpressionExecutionContext,
) {
  let examples = vec![
    ("$json.user.labels.length()", "标签数量"),
    ("$json.user.labels.first()", "第一个标签"),
    ("$json.user.labels.last()", "最后一个标签"),
    ("$json.user.labels.join(\", \")", "标签连接"),
    ("$json.user.labels.sort()", "标签排序"),
    ("$json.user.labels.reverse()", "标签反转"),
    ("$json.orders.map(\"amount\")", "提取所有订单金额"),
    ("$json.orders.filter(\"status\")", "过滤有状态的订单"),
  ];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {} -> {:?}", desc, result),
      Err(e) => println!("  {} -> 错误: {:?}", desc, e),
    }
  }
}

fn demo_context_access(
  evaluator: &ExpressionEvaluator,
  proxy: &DefaultDataProxy,
  context: &ExpressionExecutionContext,
) {
  let examples = vec![
    ("$workflow.name", "工作流名称"),
    ("$workflow.id", "工作流ID"),
    ("$execution.mode", "执行模式"),
    ("$env[\"API_KEY\"]", "环境变量 API_KEY"),
    ("$env[\"DEBUG\"]", "环境变量 DEBUG"),
    ("$vars[\"threshold\"]", "自定义变量 threshold"),
    ("$vars[\"feature_flags\"]", "自定义变量 feature_flags"),
    ("$http.pagination.page", "HTTP 分页页码"),
    ("$http.pagination.total", "HTTP 分页总数"),
    ("$http.pagination.has_next", "是否有下一页"),
  ];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {} -> {:?}", desc, result),
      Err(e) => println!("  {} -> 错误: {:?}", desc, e),
    }
  }
}

fn demo_node_access(evaluator: &ExpressionEvaluator, proxy: &DefaultDataProxy, context: &ExpressionExecutionContext) {
  let examples = vec![("$(\"database_query\")", "获取数据库查询节点输出"), ("$(\"api_call\")", "获取API调用节点输出")];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {} -> {:?}", desc, result),
      Err(e) => println!("  {} -> 错误: {:?}", desc, e),
    }
  }
}

fn demo_complex_expressions(
  evaluator: &ExpressionEvaluator,
  proxy: &DefaultDataProxy,
  context: &ExpressionExecutionContext,
) {
  let examples = vec![
    ("$json.user.profile.score > $vars[\"threshold\"] ? \"通过\" : \"未通过\"", "基于阈值的分数判断"),
    ("\"用户 \" + $json.user.name + \" 的分数是 \" + $json.user.profile.score", "动态字符串构建"),
    ("$json.orders[0].amount + $json.orders[1].amount", "计算总订单金额"),
    ("$json.user.age > 25 && $json.user.profile.score > 90 ? \"高级用户\" : \"普通用户\"", "复合条件用户分类"),
    ("$json.user.labels.length() > 2 ? $json.user.labels.join(\", \") : \"标签较少\"", "条件性数组操作"),
  ];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {} -> {:?}", desc, result),
      Err(e) => println!("  {} -> 错误: {:?}", desc, e),
    }
  }
}

fn demo_javascript_expressions(
  evaluator: &ExpressionEvaluator,
  proxy: &DefaultDataProxy,
  context: &ExpressionExecutionContext,
) {
  let examples = vec![
    ("{{$json.user.age * 12}}", "JavaScript 风格：年龄转换为月数"),
    ("{{$json.user.name.split(\"\").length()}}", "JavaScript 风格：计算姓名字符数"),
    ("={{$json.user.profile.score >= 90}}", "表达式前缀：分数判断"),
  ];

  for (expr, desc) in examples {
    match evaluator.evaluate(expr, proxy, context) {
      Ok(result) => println!("  {} -> {:?}", desc, result),
      Err(e) => println!("  {} -> 错误: {:?}", desc, e),
    }
  }
}
