import { HetuflowClient } from "../utils/client.js";
import {
  AgentForCreate,
  AgentForUpdate,
  AgentForQuery,
  SchedAgent,
  PageResult_SchedAgent,
  IdStringResult,
} from "../types/index.js";

export class AgentAPI {
  constructor(private client: HetuflowClient) {}

  /**
   * 创建 Agent
   */
  async createAgent(data: AgentForCreate): Promise<IdStringResult> {
    return this.client.post<IdStringResult>("/api/v1/agents/create", data);
  }

  /**
   * 查询 Agent 列表
   */
  async queryAgents(query: AgentForQuery): Promise<PageResult_SchedAgent> {
    return this.client.post<PageResult_SchedAgent>("/api/v1/agents/query", query);
  }

  /**
   * 获取单个 Agent
   */
  async getAgent(id: string): Promise<SchedAgent | null> {
    return this.client.get<SchedAgent | null>(`/api/v1/agents/${id}`);
  }

  /**
   * 更新 Agent
   */
  async updateAgent(id: string, data: AgentForUpdate): Promise<void> {
    return this.client.post<void>(`/api/v1/agents/${id}/update`, data);
  }

  /**
   * 删除 Agent
   */
  async deleteAgent(id: string): Promise<void> {
    return this.client.delete<void>(`/api/v1/agents/${id}`);
  }
}
