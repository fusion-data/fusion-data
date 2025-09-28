import { HetuflowClient } from '../utils/client.js';
import {
  TaskForCreate,
  TaskForUpdate,
  TaskForQuery,
  SchedTask,
  PageResult_SchedTask,
  IdUuidResult,
} from '../types/index.js';

export class TaskAPI {
  constructor(private client: HetuflowClient) {}

  /**
   * 创建 Task
   */
  async createTask(data: TaskForCreate): Promise<IdUuidResult> {
    return this.client.post<IdUuidResult>('/api/v1/tasks/create', data);
  }

  /**
   * 查询 Task 列表
   */
  async queryTasks(query: TaskForQuery): Promise<PageResult_SchedTask> {
    return this.client.post<PageResult_SchedTask>('/api/v1/tasks/query', query);
  }

  /**
   * 获取单个 Task
   */
  async getTask(id: string): Promise<SchedTask | null> {
    return this.client.get<SchedTask | null>(`/api/v1/tasks/${id}`);
  }

  /**
   * 更新 Task
   */
  async updateTask(id: string, data: TaskForUpdate): Promise<void> {
    return this.client.post<void>(`/api/v1/tasks/${id}/update`, data);
  }

  /**
   * 删除 Task
   */
  async deleteTask(id: string): Promise<void> {
    return this.client.delete<void>(`/api/v1/tasks/${id}`);
  }

  /**
   * 取消 Task
   */
  async cancelTask(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/tasks/${id}/cancel`);
  }

  /**
   * 重试 Task
   */
  async retryTask(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/tasks/${id}/retry`);
  }
}
