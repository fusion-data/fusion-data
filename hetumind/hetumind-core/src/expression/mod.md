# Hetumind 表达式系统

这是一个受 n8n 启发的 Rust 表达式系统实现，支持动态数据访问、函数调用、方法链等功能。

## 功能特性

### 1. 数据访问 & 引用

- `$json` - 访问当前节点的 JSON 数据
- `$binary` - 访问二进制数据
- `$("NodeName").first()` - 访问其它节点的第一个输出
- `$("NodeName").last()` - 访问其它节点的最后一个输出
- `$("NodeName").all()` - 访问其它节点的所有输出
- `$("NodeName").outputs(index)` - 访问其它节点的指定索引输出
- `$input.all()` - 获取所有输入项
- `$input.first()` - 获取第一个输入项
- `$input.last()` - 获取最后一个输入项
- `$input.item` - 获取当前处理的输入项
- `$input.items(index)` - 获取指定索引的输入项

### 2. 时间处理

- `$now` - 当前时间（yyyy-MM-ddTHH:mm:ss.Z）
- `$today` - 今天日期（00:00:00）
- `$now.plus({days: 7})` - 时间加法
- `$now.toFormat("yyyy-MM-dd HH:mm:ss")` - 格式化时间

### 3. JSON 查询

- `$jsonpath("$.items[?(@.status=='active')]", $json)` - JSONPath 查询

### 4. 字符串处理

- `.toUpperCase()` - 转大写
- `.toLowerCase()` - 转小写
- `.trim()` - 去除首尾空格
- `.split(delimiter)` - 分割字符串
- `.replaceSpecialChars(replacement)` - 替换特殊字符
- `.toTitleCase()` - 转换为标题格式
- `.extractEmail()` - 提取邮箱地址

### 5. 数组操作

- `.length` - 获取长度
- `.join(separator)` - 连接数组元素
- `.filter(condition)` - 过滤数组（简化实现）
- `[index]` - 索引访问

### 6. 条件 & 数学

- `$if(condition, then_value, else_value)` - 条件函数
- `$ifEmpty(value, defaultValue)` - 空值处理
- `$max(a, b, ...)` - 最大值
- `$min(a, b, ...)` - 最小值
- `condition ? then : else` - 三元运算符

### 7. 环境元数据

- `$workflow.name` - 工作流名称
- `$workflow.id` - 工作流 ID
- `$workflow.active` - 工作流是否激活
- `$execution.id` - 执行 ID
- `$execution.mode` - 执行模式
- `$env["VARIABLE_NAME"]` - 环境变量
- `$vars.custom_var` - 自定义变量

### 8. HTTP 分页

- `$http.pagination.page` - 当前页码
- `$http.pagination.total` - 总记录数
- `$http.pagination.per_page` - 每页记录数
- `$http.pagination.has_next` - 是否有下一页

## 使用示例

```rust
use hetumind_core::expression::{
    DefaultDataProxy, ExecutionContext, ExpressionEvaluator, Value,
};
use std::collections::HashMap;

// 创建评估器
let evaluator = ExpressionEvaluator::new();

// 创建执行上下文
let context = ExecutionContext::new("workflow-1".to_string(), "My Workflow".to_string());

// 创建数据
let data = Value::Object(HashMap::from_iter([
    ("name".to_string(), Value::String("John".to_string())),
    ("age".to_string(), Value::Number(30.0)),
]));

// 创建数据代理
let proxy = DefaultDataProxy::new(data);

// 评估表达式
let result = evaluator.evaluate("$json.name.toUpperCase()", &proxy, &context)?;
assert_eq!(result, Value::String("JOHN".to_string()));
```

## 表达式语法

### 基本语法

- 变量访问：`$variable`
- 属性访问：`object.property`
- 方法调用：`object.method(args)`
- 索引访问：`array[index]` 或 `object["key"]`
- 函数调用：`$function(args)`

### 操作符

- 算术：`+`, `-`, `*`, `/`, `%`
- 比较：`==`, `!=`, `<`, `>`, `<=`, `>=`
- 逻辑：`&&`, `||`
- 条件：`condition ? then : else`

### 链式调用

支持属性访问和方法调用的链式操作：

```
$json.users[0].profile.email.toLowerCase().split('@')[0]
```

## 待实现功能

### JavaScript 表达式

计划支持 `{{ ... }}` 语法来执行更复杂的 JavaScript 表达式：

```
{{ $json.items.map(item => item.price * 0.9).reduce((a, b) => a + b, 0) }}
```

### 更多数组方法

- `.map(fn)` - 映射数组
- `.reduce(fn, initial)` - 归约数组
- `.find(condition)` - 查找元素
- `.some(condition)` - 检查是否有元素满足条件
- `.every(condition)` - 检查是否所有元素满足条件

### 更多字符串方法

- `.replace(search, replacement)` - 替换字符串
- `.substring(start, end)` - 截取子串
- `.indexOf(search)` - 查找子串位置
- `.includes(search)` - 检查是否包含子串

## 扩展系统

可以通过 `FunctionRegistry` 注册自定义函数：

```rust
let mut registry = FunctionRegistry::new();
registry.register("myFunction", |args, proxy, context| {
    // 自定义函数实现
    Ok(Value::String("result".to_string()))
});
```

## 错误处理

系统提供了详细的错误信息：

- `ParseError` - 解析错误
- `EvaluationError` - 评估错误
- `FunctionError` - 函数执行错误
- `TypeError` - 类型错误

所有错误都包含了有用的上下文信息，便于调试。
