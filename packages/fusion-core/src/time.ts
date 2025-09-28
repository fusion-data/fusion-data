/**
 * 示例工具函数：格式化日期
 */
export function formatDate(date: Date): string {
  return date.toISOString().split('T')[0];
}

/**
 * 示例工具函数：延迟执行
 */
export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
