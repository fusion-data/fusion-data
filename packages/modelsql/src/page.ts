/**
 * 排序方向枚举
 */
export type OrderDirection = 'asc' | 'desc';

// region: --- Page 接口定义

/**
 * 分页参数接口
 * 对应 Rust 的 Page 结构体，使用 snake_case 字段名
 */
export interface Page {
  // 指定返回的页码
  page?: number;
  // 指定返回的条数
  limit?: number;
  // 指定返回的偏移量
  offset?: number;
  // 指定返回的排序
  // `!field`: `field desc`, `field`: `field asc`
  order_bys?: string[];
}

/**
 * 分页信息接口
 * 对应 Rust 的 Paged 结构体
 */
export interface Paged {
  total: number;
}

/**
 * 分页结果接口
 * 对应 Rust 的 PageResult 结构体
 */
export interface PageResult<T> {
  page: Paged;
  result: T[];
}

// endregion: --- Page 接口定义

// region: --- 工具函数

/**
 * 创建只包含 limit 的分页对象
 *
 * @param limit - 每页条数
 * @returns Page 对象
 */
export function newPageWithLimit(limit: number): Page {
  return { limit };
}

/**
 * 创建包含 offset 和 limit 的分页对象
 *
 * @param offset - 偏移量
 * @param limit - 每页条数
 * @returns Page 对象
 */
export function newPageWithOffsetLimit(offset: number, limit: number): Page {
  return { offset, limit };
}

/**
 * 计算分页的偏移量
 * 优先使用 offset，如果没有则根据 page 和 limit 计算
 *
 * @param page - 分页对象
 * @returns 偏移量，如果无法计算则返回 undefined
 */
export function getPageOffset(page: Page): number | undefined {
  if (page.offset !== undefined) {
    return page.offset;
  }

  if (page.page !== undefined && page.limit !== undefined) {
    return (page.page - 1) * page.limit;
  }

  return undefined;
}

/**
 * 创建分页信息对象
 *
 * @param total - 总记录数
 * @returns Paged 对象
 */
export function createPaged(total: number): Paged {
  return { total };
}

/**
 * 创建分页结果对象
 *
 * @param total - 总记录数
 * @param result - 结果数组
 * @returns PageResult 对象
 */
export function createPageResult<T>(total: number, result: T[]): PageResult<T> {
  return {
    page: { total },
    result,
  };
}

// endregion: --- 工具函数
