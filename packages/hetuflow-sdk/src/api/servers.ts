import { HetuflowClient } from '../utils/client.js';
import {
  ServerForUpdate,
  ServerForQuery,
  SchedServer,
  PageResult_SchedServer,
  IdStringResult,
} from '../types/index.js';

export class ServerAPI {
  constructor(private client: HetuflowClient) {}

  /**
   * 查询 Server 列表
   */
  async queryServers(query: ServerForQuery): Promise<PageResult_SchedServer> {
    return this.client.post<PageResult_SchedServer>('/api/v1/servers/query', query);
  }

  /**
   * 获取单个 Server
   */
  async getServer(id: string): Promise<SchedServer | null> {
    return this.client.get<SchedServer | null>(`/api/v1/servers/${id}`);
  }

  /**
   * 更新 Server
   */
  async updateServer(id: string, data: ServerForUpdate): Promise<void> {
    return this.client.post<void>(`/api/v1/servers/${id}/update`, data);
  }

  /**
   * 删除 Server
   */
  async deleteServer(id: string): Promise<void> {
    return this.client.delete<void>(`/api/v1/servers/${id}`);
  }
}
