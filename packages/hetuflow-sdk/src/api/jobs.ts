import { HetuflowClient } from "../utils/client.js";
import {
  JobForCreate,
  JobForUpdate,
  JobForQuery,
  SchedJob,
  PageResult_SchedJob,
  IdUuidResult,
} from "../types/index.js";

export class JobAPI {
  constructor(private client: HetuflowClient) {}

  /**
   * 创建 Job
   */
  async createJob(data: JobForCreate): Promise<IdUuidResult> {
    return this.client.post<IdUuidResult>("/api/v1/jobs/item", data);
  }

  /**
   * 查询 Job 列表
   */
  async queryJobs(query: JobForQuery): Promise<PageResult_SchedJob> {
    return this.client.post<PageResult_SchedJob>("/api/v1/jobs/page", query);
  }

  /**
   * 获取单个 Job
   */
  async getJob(id: string): Promise<SchedJob | null> {
    return this.client.get<SchedJob | null>(`/api/v1/jobs/item/${id}`);
  }

  /**
   * 更新 Job
   */
  async updateJob(id: string, data: JobForUpdate): Promise<void> {
    return this.client.put<void>(`/api/v1/jobs/item/${id}`, data);
  }

  /**
   * 删除 Job
   */
  async deleteJob(id: string): Promise<void> {
    return this.client.delete<void>(`/api/v1/jobs/item/${id}`);
  }

  /**
   * 启用 Job
   */
  async enableJob(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/jobs/item/${id}/enable`);
  }

  /**
   * 禁用 Job
   */
  async disableJob(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/jobs/item/${id}/disable`);
  }
}
