import { HetuflowSDK } from "@fusion-data/hetuflow-sdk";

/**
 * HetuFlow API 服务配置
 */
class APIService {
  private sdk: HetuflowSDK;

  constructor() {
    // 初始化 SDK，从环境变量或配置中获取 baseURL
    this.sdk = new HetuflowSDK({
      baseURL: import.meta.env.VITE_API_BASE_URL || "",
      timeout: 30000,
    });
  }

  /**
   * 设置认证 Token
   */
  setToken(token: string): void {
    this.sdk.setToken(token);
  }

  /**
   * 清除认证 Token
   */
  clearToken(): void {
    this.sdk.clearToken();
  }

  /**
   * 获取 SDK 实例
   */
  getSDK(): HetuflowSDK {
    return this.sdk;
  }

  // 代理相关 API
  get agents() {
    return this.sdk.agents;
  }

  // 服务器相关 API
  get servers() {
    return this.sdk.servers;
  }

  // 作业相关 API
  get jobs() {
    return this.sdk.jobs;
  }

  // 任务相关 API
  get tasks() {
    return this.sdk.tasks;
  }

  // 任务实例相关 API
  get taskInstances() {
    return this.sdk.taskInstances;
  }

  // 认证相关 API
  get auth() {
    return this.sdk.auth;
  }

  // 网关相关 API
  get gateway() {
    return this.sdk.gateway;
  }

  // 系统相关 API
  get system() {
    return this.sdk.system;
  }
}

// 创建全局 API 服务实例
export const apiService = new APIService();

// 导出类型定义
export * from "@fusion-data/hetuflow-sdk";
