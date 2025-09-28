import { HetuflowClient, HetuflowClientConfig } from './utils/client.js';
import { AgentAPI } from './api/agents.js';
import { ServerAPI } from './api/servers.js';
import { JobAPI } from './api/jobs.js';
import { TaskAPI } from './api/tasks.js';
import { TaskInstanceAPI } from './api/task-instances.js';
import { AuthAPI } from './api/auth.js';
import { GatewayAPI } from './api/gateway.js';
import { SystemAPI } from './api/system.js';

export class HetuflowSDK {
  private client: HetuflowClient;

  public readonly agents: AgentAPI;
  public readonly servers: ServerAPI;
  public readonly jobs: JobAPI;
  public readonly tasks: TaskAPI;
  public readonly taskInstances: TaskInstanceAPI;
  public readonly auth: AuthAPI;
  public readonly gateway: GatewayAPI;
  public readonly system: SystemAPI;

  constructor(config: HetuflowClientConfig) {
    this.client = new HetuflowClient(config);

    // 初始化各个 API 模块
    this.agents = new AgentAPI(this.client);
    this.servers = new ServerAPI(this.client);
    this.jobs = new JobAPI(this.client);
    this.tasks = new TaskAPI(this.client);
    this.taskInstances = new TaskInstanceAPI(this.client);
    this.auth = new AuthAPI(this.client);
    this.gateway = new GatewayAPI(this.client);
    this.system = new SystemAPI(this.client);
  }

  /**
   * 设置认证 Token
   */
  setToken(token: string): void {
    this.client.setToken(token);
  }

  /**
   * 清除认证 Token
   */
  clearToken(): void {
    this.client.clearToken();
  }

  /**
   * 创建一个新的 SDK 实例
   */
  static create(config: HetuflowClientConfig): HetuflowSDK {
    return new HetuflowSDK(config);
  }
}

// 默认导出
export default HetuflowSDK;
