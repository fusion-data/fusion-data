use super::Value;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
  #[error("语法错误: {message} at position {position}")]
  SyntaxError { message: String, position: usize },
  #[error("未预期的令牌: {token}")]
  UnexpectedToken { token: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
  Literal(Value),
  Variable(String),
  PropertyAccess { object: Box<Expression>, property: String },
  IndexAccess { object: Box<Expression>, index: Box<Expression> },
  MethodCall { object: Box<Expression>, method: String, args: Vec<Expression> },
  FunctionCall { name: String, args: Vec<Expression> },
  BinaryOp { left: Box<Expression>, operator: BinaryOperator, right: Box<Expression> },
  ConditionalExpr { condition: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression> },
  JsonPath { path: String, data: Box<Expression> },
  NodeAccess { node_name: String },
  InputAccess { method: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Equal,
  NotEqual,
  Less,
  Greater,
  LessEqual,
  GreaterEqual,
  And,
  Or,
}

pub struct ExpressionParser;

impl ExpressionParser {
  pub fn parse(input: &str) -> Result<Expression, ParseError> {
    let input = input.trim();

    // 检查是否是简单表达式（以 = 开头）
    let input = if let Some(stripped) = input.strip_prefix('=') { stripped } else { input };

    // 检查是否是 JavaScript 表达式（{{ ... }}）
    if input.starts_with("{{") && input.ends_with("}}") {
      let js_expr = &input[2..input.len() - 2].trim();
      return Self::parse(js_expr);
    }

    Self::parse_expression(input)
  }

  fn parse_expression(input: &str) -> Result<Expression, ParseError> {
    let input = input.trim();

    // 处理条件表达式 (? :)
    if let Some((condition, then_expr, else_expr)) = Self::find_conditional_operator(input) {
      return Ok(Expression::ConditionalExpr {
        condition: Box::new(Self::parse_expression(condition)?),
        then_expr: Box::new(Self::parse_expression(then_expr)?),
        else_expr: Box::new(Self::parse_expression(else_expr)?),
      });
    }

    // 处理二元运算符（从低优先级到高优先级）
    for ops in &[
      vec!["||"],
      vec!["&&"],
      vec!["==", "!="],
      vec!["<=", ">=", "<", ">"],
      vec!["+", "-"],
      vec!["*", "/", "%"],
    ] {
      if let Some((left, op, right)) = Self::find_binary_operator(input, ops) {
        let operator = match op {
          "||" => BinaryOperator::Or,
          "&&" => BinaryOperator::And,
          "==" => BinaryOperator::Equal,
          "!=" => BinaryOperator::NotEqual,
          "<=" => BinaryOperator::LessEqual,
          ">=" => BinaryOperator::GreaterEqual,
          "<" => BinaryOperator::Less,
          ">" => BinaryOperator::Greater,
          "+" => BinaryOperator::Add,
          "-" => BinaryOperator::Sub,
          "*" => BinaryOperator::Mul,
          "/" => BinaryOperator::Div,
          "%" => BinaryOperator::Mod,
          _ => return Err(ParseError::SyntaxError { message: format!("未知运算符: {}", op), position: 0 }),
        };

        return Ok(Expression::BinaryOp {
          left: Box::new(Self::parse_expression(left)?),
          operator,
          right: Box::new(Self::parse_expression(right)?),
        });
      }
    }

    // 处理括号表达式 (可能带方法调用)
    if input.starts_with('(')
      && let Some(closing_paren) = input.find(')')
    {
      if closing_paren + 1 < input.len() {
        // 括号后还有内容，可能是方法调用
        let paren_expr = &input[1..closing_paren];
        let method_part = &input[closing_paren + 1..];

        let mut expr = Self::parse_expression(paren_expr)?;

        // 移除点号
        if let Some(method_part) = method_part.strip_prefix('.') {
          if method_part.contains('(') && method_part.ends_with(')') {
            if let Some(paren_pos) = method_part.find('(') {
              let method_name = &method_part[..paren_pos];
              let args_str = &method_part[paren_pos + 1..method_part.len() - 1];
              let args = if args_str.trim().is_empty() { Vec::new() } else { Self::parse_function_args(args_str)? };

              expr = Expression::MethodCall { object: Box::new(expr), method: method_name.to_string(), args };
            }
          } else {
            expr = Expression::PropertyAccess { object: Box::new(expr), property: method_part.to_string() };
          }
        }

        return Ok(expr);
      } else if closing_paren + 1 == input.len() {
        // 简单的括号表达式
        let inner_expr = &input[1..closing_paren];
        return Self::parse_expression(inner_expr);
      }
    }

    // 处理基本表达式
    Self::parse_primary(input)
  }

  fn find_conditional_operator(input: &str) -> Option<(&str, &str, &str)> {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut question_byte_pos = None;
    let mut colon_byte_pos = None;
    let mut byte_pos = 0;

    for ch in input.chars() {
      match ch {
        '(' => paren_depth += 1,
        ')' => paren_depth -= 1,
        '[' => bracket_depth += 1,
        ']' => bracket_depth -= 1,
        '?' if paren_depth == 0 && bracket_depth == 0 && question_byte_pos.is_none() => {
          question_byte_pos = Some(byte_pos);
        }
        ':' if paren_depth == 0 && bracket_depth == 0 && question_byte_pos.is_some() && colon_byte_pos.is_none() => {
          colon_byte_pos = Some(byte_pos);
          break;
        }
        _ => {}
      }
      byte_pos += ch.len_utf8();
    }

    if let (Some(q_pos), Some(c_pos)) = (question_byte_pos, colon_byte_pos) {
      let condition = input[..q_pos].trim();
      let then_expr = input[q_pos + 1..c_pos].trim();
      let else_expr = input[c_pos + 1..].trim();
      Some((condition, then_expr, else_expr))
    } else {
      None
    }
  }

  fn find_binary_operator<'a>(input: &'a str, operators: &[&'a str]) -> Option<(&'a str, &'a str, &'a str)> {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_string = false;
    let mut string_char = '\0';

    // 从右到左扫描字节位置
    let mut byte_positions = Vec::new();
    let mut byte_pos = 0;
    for ch in input.chars() {
      byte_positions.push(byte_pos);
      byte_pos += ch.len_utf8();
    }
    byte_positions.push(byte_pos); // 添加最后一个位置

    let chars: Vec<char> = input.chars().collect();

    // 从右到左扫描（右结合）
    for i in (0..chars.len()).rev() {
      match chars[i] {
        '"' | '\'' if !in_string => {
          in_string = true;
          string_char = chars[i];
        }
        ch if in_string && ch == string_char => {
          in_string = false;
        }
        ')' if !in_string => paren_depth += 1,
        '(' if !in_string => paren_depth -= 1,
        ']' if !in_string => bracket_depth += 1,
        '[' if !in_string => bracket_depth -= 1,
        _ => {}
      }

      if !in_string && paren_depth == 0 && bracket_depth == 0 {
        for &op in operators {
          if i + op.len() <= chars.len() {
            let slice: String = chars[i..i + op.len()].iter().collect();
            if slice == op {
              let left_end = byte_positions[i];
              let right_start = byte_positions[i + op.len()];
              let left = &input[..left_end];
              let right = &input[right_start..];

              // 特殊处理：如果是减号且左边为空，则不作为二元运算符处理
              if op == "-" && left.trim().is_empty() {
                continue;
              }

              return Some((left.trim(), op, right.trim()));
            }
          }
        }
      }
    }

    None
  }

  fn parse_primary(input: &str) -> Result<Expression, ParseError> {
    let input = input.trim();

    // 节点访问 $("NodeName")
    if input.starts_with("$(") && input.ends_with(')') {
      let inner = &input[2..input.len() - 1].trim();
      if inner.starts_with('"') && inner.ends_with('"') {
        let node_name = &inner[1..inner.len() - 1];
        return Ok(Expression::NodeAccess { node_name: node_name.to_string() });
      }
    }

    // 函数调用 $func(args) - 只匹配不包含点号的简单函数调用
    if input.starts_with('$')
      && input.contains('(')
      && input.ends_with(')')
      && !input.contains('.')
      && let Some(paren_pos) = input.find('(')
    {
      let func_name = &input[..paren_pos];
      let args_str = &input[paren_pos + 1..input.len() - 1];
      let args = if args_str.trim().is_empty() { Vec::new() } else { Self::parse_function_args(args_str)? };

      return Ok(Expression::FunctionCall { name: func_name.to_string(), args });
    }

    // 处理复杂的属性访问和方法调用
    if input.starts_with('$') {
      return Self::parse_variable_or_access(input);
    }

    // 对象字面量
    if input.starts_with('{') && input.ends_with('}') {
      return Self::parse_object_literal(input);
    }

    // 数组字面量
    if input.starts_with('[') && input.ends_with(']') {
      return Self::parse_array_literal(input);
    }

    // 字符串字面量
    if ((input.starts_with('"') && input.ends_with('"')) || (input.starts_with('\'') && input.ends_with('\'')))
      && input.len() >= 2
    {
      let content = &input[1..input.len() - 1];
      return Ok(Expression::Literal(Value::String(content.to_string())));
    }

    // 数字字面量 (可能带方法调用)
    if let Some(dot_pos) = input.find('.') {
      let number_part = &input[..dot_pos];
      if let Ok(num) = number_part.parse::<f64>() {
        let method_part = &input[dot_pos + 1..];

        // 检查是否是小数部分
        if method_part.chars().all(|c| c.is_ascii_digit()) {
          // 这是一个小数，直接解析整个数字
          if let Ok(full_num) = input.parse::<f64>() {
            return Ok(Expression::Literal(Value::Number(full_num)));
          }
        }

        // 这是一个数字后跟方法调用的情况，递归解析
        let mut expr = Expression::Literal(Value::Number(num));

        if method_part.contains('(') && method_part.ends_with(')') {
          if let Some(paren_pos) = method_part.find('(') {
            let method_name = &method_part[..paren_pos];
            let args_str = &method_part[paren_pos + 1..method_part.len() - 1];
            let args = if args_str.trim().is_empty() { Vec::new() } else { Self::parse_function_args(args_str)? };

            expr = Expression::MethodCall { object: Box::new(expr), method: method_name.to_string(), args };
          }
        } else {
          expr = Expression::PropertyAccess { object: Box::new(expr), property: method_part.to_string() };
        }

        return Ok(expr);
      }
    }

    // 简单数字字面量（包括负数）
    if let Ok(num) = input.parse::<f64>() {
      return Ok(Expression::Literal(Value::Number(num)));
    }

    // 布尔值
    match input {
      "true" => return Ok(Expression::Literal(Value::Bool(true))),
      "false" => return Ok(Expression::Literal(Value::Bool(false))),
      "null" => return Ok(Expression::Literal(Value::Null)),
      _ => {}
    }

    // 变量
    if input.chars().all(|c| c.is_alphanumeric() || c == '_') {
      return Ok(Expression::Variable(input.to_string()));
    }

    Err(ParseError::SyntaxError { message: format!("无法解析表达式: {input}"), position: 0 })
  }

  fn parse_variable_or_access(input: &str) -> Result<Expression, ParseError> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let chars = input.chars().peekable();
    let mut in_brackets = false;
    let mut bracket_content = String::new();
    let mut paren_depth = 0;

    for ch in chars {
      match ch {
        '(' if !in_brackets => {
          paren_depth += 1;
          current.push(ch);
        }
        ')' if !in_brackets => {
          paren_depth -= 1;
          current.push(ch);
        }
        '.' if !in_brackets && paren_depth == 0 => {
          if !current.is_empty() {
            parts.push(current.clone());
            current.clear();
          }
        }
        '[' if !in_brackets && paren_depth == 0 => {
          if !current.is_empty() {
            parts.push(current.clone());
            current.clear();
          }
          in_brackets = true;
          bracket_content.clear();
        }
        ']' if in_brackets => {
          parts.push(format!("[{}]", bracket_content));
          in_brackets = false;
          bracket_content.clear();
        }
        _ => {
          if in_brackets {
            bracket_content.push(ch);
          } else {
            current.push(ch);
          }
        }
      }
    }

    if !current.is_empty() {
      parts.push(current);
    }

    if parts.is_empty() {
      return Err(ParseError::SyntaxError { message: "空变量表达式".to_string(), position: 0 });
    }

    let mut expr = Self::parse_variable_base(&parts[0])?;

    for part in &parts[1..] {
      if part.starts_with('[') && part.ends_with(']') {
        // 索引访问
        let index_expr = &part[1..part.len() - 1];
        let index = Self::parse_expression(index_expr)?;
        expr = Expression::IndexAccess { object: Box::new(expr), index: Box::new(index) };
      } else if part.contains('(') && part.ends_with(')') {
        // 方法调用
        if let Some(paren_pos) = part.find('(') {
          let method_name = &part[..paren_pos];
          let args_str = &part[paren_pos + 1..part.len() - 1];
          let args = if args_str.trim().is_empty() { Vec::new() } else { Self::parse_function_args(args_str)? };

          expr = Expression::MethodCall { object: Box::new(expr), method: method_name.to_string(), args };
        }
      } else {
        // 属性访问
        expr = Expression::PropertyAccess { object: Box::new(expr), property: part.to_string() };
      }
    }

    Ok(expr)
  }

  fn parse_variable_base(var: &str) -> Result<Expression, ParseError> {
    if var.is_empty() {
      return Err(ParseError::SyntaxError { message: "空变量名".to_string(), position: 0 });
    }

    match var {
      "$json" => Ok(Expression::Variable("json".to_string())),
      "$binary" => Ok(Expression::Variable("binary".to_string())),
      "$now" => Ok(Expression::Variable("now".to_string())),
      "$today" => Ok(Expression::Variable("today".to_string())),
      "$workflow" => Ok(Expression::Variable("workflow".to_string())),
      "$execution" => Ok(Expression::Variable("execution".to_string())),
      "$env" => Ok(Expression::Variable("env".to_string())),
      "$vars" => Ok(Expression::Variable("vars".to_string())),
      "$http" => Ok(Expression::Variable("http".to_string())),
      "$input" => Ok(Expression::Variable("input".to_string())),
      _ => Err(ParseError::SyntaxError { message: format!("未知变量: '{}'", var), position: 0 }),
    }
  }

  fn parse_function_args(args_str: &str) -> Result<Vec<Expression>, ParseError> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_string = false;
    let mut string_char = '\0';

    for ch in args_str.chars() {
      match ch {
        '"' | '\'' if !in_string => {
          in_string = true;
          string_char = ch;
          current_arg.push(ch);
        }
        ch if in_string && ch == string_char => {
          in_string = false;
          current_arg.push(ch);
        }
        '(' if !in_string => {
          paren_depth += 1;
          current_arg.push(ch);
        }
        ')' if !in_string => {
          paren_depth -= 1;
          current_arg.push(ch);
        }
        '[' if !in_string => {
          bracket_depth += 1;
          current_arg.push(ch);
        }
        ']' if !in_string => {
          bracket_depth -= 1;
          current_arg.push(ch);
        }
        ',' if !in_string && paren_depth == 0 && bracket_depth == 0 => {
          args.push(Self::parse_expression(current_arg.trim())?);
          current_arg.clear();
        }
        _ => {
          current_arg.push(ch);
        }
      }
    }

    if !current_arg.trim().is_empty() {
      args.push(Self::parse_expression(current_arg.trim())?);
    }

    Ok(args)
  }

  fn parse_object_literal(input: &str) -> Result<Expression, ParseError> {
    use std::collections::HashMap;

    let content = &input[1..input.len() - 1].trim();
    if content.is_empty() {
      return Ok(Expression::Literal(Value::Object(HashMap::default())));
    }

    let mut object = HashMap::default();
    let mut current_pair = String::new();
    let mut brace_depth = 0;
    let mut bracket_depth = 0;
    let mut in_string = false;
    let mut string_char = '\0';

    for ch in content.chars() {
      match ch {
        '"' | '\'' if !in_string => {
          in_string = true;
          string_char = ch;
          current_pair.push(ch);
        }
        ch if in_string && ch == string_char => {
          in_string = false;
          current_pair.push(ch);
        }
        '{' if !in_string => {
          brace_depth += 1;
          current_pair.push(ch);
        }
        '}' if !in_string => {
          brace_depth -= 1;
          current_pair.push(ch);
        }
        '[' if !in_string => {
          bracket_depth += 1;
          current_pair.push(ch);
        }
        ']' if !in_string => {
          bracket_depth -= 1;
          current_pair.push(ch);
        }
        ',' if !in_string && brace_depth == 0 && bracket_depth == 0 => {
          if let Some((key, value)) = Self::parse_key_value_pair(current_pair.trim())? {
            object.insert(key, value);
          }
          current_pair.clear();
        }
        _ => {
          current_pair.push(ch);
        }
      }
    }

    if !current_pair.trim().is_empty()
      && let Some((key, value)) = Self::parse_key_value_pair(current_pair.trim())?
    {
      object.insert(key, value);
    }

    Ok(Expression::Literal(Value::Object(object)))
  }

  fn parse_key_value_pair(pair: &str) -> Result<Option<(String, Value)>, ParseError> {
    if let Some(colon_pos) = pair.find(':') {
      let key_part = &pair[..colon_pos].trim();
      let value_part = &pair[colon_pos + 1..].trim();

      // 解析键（移除引号）
      let key = if (key_part.starts_with('"') && key_part.ends_with('"'))
        || (key_part.starts_with('\'') && key_part.ends_with('\''))
      {
        key_part[1..key_part.len() - 1].to_string()
      } else {
        key_part.to_string()
      };

      // 解析值
      let value = Self::parse_literal_value(value_part)?;
      Ok(Some((key, value)))
    } else {
      Ok(None)
    }
  }

  fn parse_array_literal(input: &str) -> Result<Expression, ParseError> {
    let content = &input[1..input.len() - 1].trim();
    if content.is_empty() {
      return Ok(Expression::Literal(Value::Array(Vec::new())));
    }

    let mut array = Vec::new();
    let mut current_item = String::new();
    let mut brace_depth = 0;
    let mut bracket_depth = 0;
    let mut in_string = false;
    let mut string_char = '\0';

    for ch in content.chars() {
      match ch {
        '"' | '\'' if !in_string => {
          in_string = true;
          string_char = ch;
          current_item.push(ch);
        }
        ch if in_string && ch == string_char => {
          in_string = false;
          current_item.push(ch);
        }
        '{' if !in_string => {
          brace_depth += 1;
          current_item.push(ch);
        }
        '}' if !in_string => {
          brace_depth -= 1;
          current_item.push(ch);
        }
        '[' if !in_string => {
          bracket_depth += 1;
          current_item.push(ch);
        }
        ']' if !in_string => {
          bracket_depth -= 1;
          current_item.push(ch);
        }
        ',' if !in_string && brace_depth == 0 && bracket_depth == 0 => {
          let value = Self::parse_literal_value(current_item.trim())?;
          array.push(value);
          current_item.clear();
        }
        _ => {
          current_item.push(ch);
        }
      }
    }

    if !current_item.trim().is_empty() {
      let value = Self::parse_literal_value(current_item.trim())?;
      array.push(value);
    }

    Ok(Expression::Literal(Value::Array(array)))
  }

  fn parse_literal_value(input: &str) -> Result<Value, ParseError> {
    let input = input.trim();

    // 字符串
    if (input.starts_with('"') && input.ends_with('"')) || (input.starts_with('\'') && input.ends_with('\'')) {
      let content = &input[1..input.len() - 1];
      return Ok(Value::String(content.to_string()));
    }

    // 数字
    if let Ok(num) = input.parse::<f64>() {
      return Ok(Value::Number(num));
    }

    // 布尔值
    match input {
      "true" => return Ok(Value::Bool(true)),
      "false" => return Ok(Value::Bool(false)),
      "null" => return Ok(Value::Null),
      _ => {}
    }

    // 对象
    if input.starts_with('{')
      && input.ends_with('}')
      && let Ok(Expression::Literal(value)) = Self::parse_object_literal(input)
    {
      return Ok(value);
    }

    // 数组
    if input.starts_with('[')
      && input.ends_with(']')
      && let Ok(Expression::Literal(value)) = Self::parse_array_literal(input)
    {
      return Ok(value);
    }

    Err(ParseError::SyntaxError { message: format!("无法解析字面量: {}", input), position: 0 })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_literal() {
    let result = ExpressionParser::parse("42").unwrap();
    assert_eq!(result, Expression::Literal(Value::Number(42.0)));

    let result = ExpressionParser::parse("\"hello\"").unwrap();
    assert_eq!(result, Expression::Literal(Value::String("hello".to_string())));

    let result = ExpressionParser::parse("true").unwrap();
    assert_eq!(result, Expression::Literal(Value::Bool(true)));
  }

  #[test]
  fn test_parse_variable() {
    let result = ExpressionParser::parse("$json").unwrap();
    assert_eq!(result, Expression::Variable("json".to_string()));

    let result = ExpressionParser::parse("$now").unwrap();
    assert_eq!(result, Expression::Variable("now".to_string()));
  }

  #[test]
  fn test_parse_property_access() {
    let result = ExpressionParser::parse("$json.name").unwrap();
    assert!(matches!(result, Expression::PropertyAccess { .. }));
  }

  #[test]
  fn test_parse_method_call() {
    let result = ExpressionParser::parse("$json.name.toUpperCase()").unwrap();
    assert!(matches!(result, Expression::MethodCall { .. }));
  }

  #[test]
  fn test_parse_binary_op() {
    let result = ExpressionParser::parse("1 + 2").unwrap();
    assert!(matches!(result, Expression::BinaryOp { .. }));
  }

  #[test]
  fn test_parse_conditional() {
    let result = ExpressionParser::parse("true ? 1 : 2").unwrap();
    assert!(matches!(result, Expression::ConditionalExpr { .. }));
  }

  #[test]
  fn test_parse_function_call() {
    let result = ExpressionParser::parse("$max(1, 2, 3)").unwrap();
    assert!(matches!(result, Expression::FunctionCall { .. }));
  }
}
