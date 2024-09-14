use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::AsRefStr;
use uuid::Uuid;

use crate::ac::Effect;

/// 策略的授权
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Policy {
  pub id: Uuid,
  pub description: Option<String>,

  /// 策略所作用的资源。
  ///
  /// 格式为 `“服务名:region:domainId:资源类型:资源路径”`, 资源类型支持通配符号*，通配符号*表示所有。示例：
  /// - `"obs:*:*:bucket:*"` 表示所有区域的 obs 桶
  /// - `"obs:*:*:bucket:mybucket/myfolder/*"` 表示 mybucket 这个桶下的所有文件
  pub resource: Vec<String>,

  /// 操作权限。
  ///
  /// 格式为“服务名:资源类型:操作”。授权项支持通配符号*，通配符号*表示所有。
  pub action: Vec<String>,

  pub effect: Effect,

  /// 使策略生效的特定条件，包括条件键和运算符。
  ///
  /// 格式为“条件运算符:{条件键：[条件值1,条件值2]}”。如果您设置多个条件，同时满足所有条件时，该策略才生效。
  ///
  /// 示例: `"StringEndWithIfExists":{"g:UserName":["specialCharacter"]}` 表示当用户输入的用户名以 `"specialCharacter"` 结尾时该条 statement 生效。
  pub condition: Option<Value>,
}

/// 策略结构
///
/// 策略结构包括Version（策略版本号）和Statement（策略权限语句）两部分，其中Statement可以有多个，表示不同的授权项。
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyStructure {
  /// 策略版本号
  pub version: PolicyVersion,

  /// 策略权限语句
  pub statement: Vec<Policy>,
}

#[derive(Debug, Default, Serialize, Deserialize, AsRefStr)]
pub enum PolicyVersion {
  #[default]
  #[strum(serialize = "v1.0")]
  V1_0,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessRequest {
  pub subject: HashMap<String, Value>,
  pub resource: Vec<String>,
  pub action: Vec<String>,
  pub environment: HashMap<String, Value>,
}
