use serde::{Deserialize, Serialize};

/* 函数级注释：分页请求模型，统一请求中的页码与每页数量 */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PageRequest {
  pub page: i32,
  pub limit: i32,
}

/* 函数级注释：分页响应模型，统一返回中的总数与是否有更多 */
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PageResponse {
  pub total: i64,
  pub has_more: bool,
}

impl PageResponse {
  /* 函数级注释：根据总数、当前页与每页数量计算是否还有更多数据 */
  pub fn new(total: i64, page: i32, limit: i32) -> Self {
    let fetched = (page.max(1) as i64 - 1) * (limit.max(1) as i64) + (limit.max(1) as i64);
    let has_more = fetched < total;
    Self { total, has_more }
  }
}

/* 函数级注释：统一的“更新时间”过滤条件（ISO8601 字符串），字段名与文档保持一致 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatedAtFilter {
  #[serde(rename = "$gte")]
  pub gte: Option<String>,
  #[serde(rename = "$lt")]
  pub lt: Option<String>,
}

/* 函数级注释：包装 UpdatedAt 过滤，filters 中的每项形如 { "updated_at": { "$gte": ..., "$lt": ... } } */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatedAtWrapper {
  pub updated_at: UpdatedAtFilter,
}

/* 函数级注释：通用变更查询请求模型，包含分页与过滤列表 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeQueryReq {
  pub page: PageRequest,
  pub filters: Vec<UpdatedAtWrapper>,
}

/* 函数级注释：通用变更查询响应模型，包含分页信息与结果列表 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeQueryResp<T> {
  pub page: PageResponse,
  pub result: Vec<T>,
}