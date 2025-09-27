/**
 * 示例工具函数：生成随机 ID
 */
export function generateId(): string {
  return Math.random().toString(36).substr(2, 9);
}
