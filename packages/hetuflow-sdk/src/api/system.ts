import { HetuflowClient } from "../utils/client.js";
import { HealthStatus } from "../types/index.js";

export class SystemAPI {
  constructor(private client: HetuflowClient) {}

  /**
   * 获取系统健康状态
   */
  async getHealth(): Promise<HealthStatus> {
    return this.client.get<HealthStatus>("/api/v1/system/health");
  }

  /**
   * 获取系统信息
   */
  async getInfo(): Promise<any> {
    return this.client.get<any>("/api/v1/system/info");
  }

  /**
   * 获取系统指标
   */
  async getMetrics(): Promise<any> {
    return this.client.get<any>("/api/v1/system/metrics");
  }
}
