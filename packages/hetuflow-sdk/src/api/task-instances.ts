import { HetuflowClient } from '../utils/client.js';
import {
  TaskInstanceForCreate,
  TaskInstanceForUpdate,
  TaskInstanceForQuery,
  SchedTaskInstance,
  PageResult_SchedTaskInstance,
} from '../types/index.js';

export class TaskInstanceAPI {
  constructor(private client: HetuflowClient) {}

  /**
   * 创建 TaskInstance
   */
  async createTaskInstance(data: TaskInstanceForCreate): Promise<string> {
    return this.client.post<string>('/api/v1/task-instances/item', data);
  }

  /**
   * 查询 TaskInstance 列表
   */
  async queryTaskInstances(query: TaskInstanceForQuery): Promise<PageResult_SchedTaskInstance> {
    return this.client.post<PageResult_SchedTaskInstance>('/api/v1/task-instances/page', query);
  }

  /**
   * 获取单个 TaskInstance
   */
  async getTaskInstance(id: string): Promise<SchedTaskInstance | null> {
    return this.client.get<SchedTaskInstance | null>(`/api/v1/task-instances/item/${id}`);
  }

  /**
   * 更新 TaskInstance
   */
  async updateTaskInstance(id: string, data: TaskInstanceForUpdate): Promise<void> {
    return this.client.post<void>(`/api/v1/task-instances/item/update`, data);
  }

  /**
   * 删除 TaskInstance
   */
  async deleteTaskInstance(id: string): Promise<void> {
    return this.client.delete<void>(`/api/v1/task-instances/item/${id}`);
  }
}
