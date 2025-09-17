import { HetuflowClient } from "../utils/client.js";
import { GatewayCommandRequest, IdUuidResult } from "../types/index.js";

export class GatewayAPI {
  constructor(private client: HetuflowClient) {}

  /**
   * 发送网关命令
   */
  async sendCommand(request: GatewayCommandRequest): Promise<IdUuidResult> {
    return this.client.post<IdUuidResult>("/api/v1/gateway/command", request);
  }

  /**
   * 获取网关状态
   */
  async getStatus(): Promise<any> {
    return this.client.get<any>("/api/v1/gateway/status");
  }

  /**
   * WebSocket 连接 URL
   */
  getWebSocketUrl(agentId: string, baseUrl?: string): string {
    const wsBaseUrl = baseUrl?.replace(/^http/, "ws") || "ws://localhost:9500";
    return `${wsBaseUrl}/api/v1/gateway/ws?agent_id=${encodeURIComponent(agentId)}`;
  }
}
