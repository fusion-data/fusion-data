import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

export interface HetumindClientConfig {
  baseURL: string;
  timeout?: number;
  headers?: Record<string, string>;
  token?: string;
}

export class HetumindClient {
  public client: AxiosInstance; // Made public for access in execution API

  constructor(config: HetumindClientConfig) {
    this.client = axios.create({
      baseURL: config.baseURL,
      timeout: config.timeout || 30000,
      headers: {
        'Content-Type': 'application/json',
        ...config.headers,
      },
    });

    // 添加请求拦截器
    this.client.interceptors.request.use(
      (config: any) => {
        // 添加认证 token
        if (this.token) {
          config.headers.Authorization = `Bearer ${this.token}`;
        }
        return config;
      },
      (error: any) => {
        return Promise.reject(error);
      }
    );

    // 添加响应拦截器
    this.client.interceptors.response.use(
      (response: any) => {
        return response;
      },
      (error: any) => {
        // 统一错误处理
        if (error.response?.data) {
          throw new HetumindError(
            error.response.data.message || 'API Error',
            error.response.status,
            error.response.data.code,
            error.response.data.details
          );
        }
        throw error;
      }
    );

    // 设置 token
    if (config.token) {
      this.setToken(config.token);
    }
  }

  private token?: string | undefined;

  setToken(token: string): void {
    this.token = token;
  }

  clearToken(): void {
    this.token = undefined;
  }

  async request<T = any>(config: AxiosRequestConfig): Promise<T> {
    const response: AxiosResponse<T> = await this.client.request(config);
    return response.data;
  }

  async get<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
    return this.request<T>({ ...config, method: 'GET', url });
  }

  async post<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    return this.request<T>({ ...config, method: 'POST', url, data });
  }

  async put<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    return this.request<T>({ ...config, method: 'PUT', url, data });
  }

  async delete<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
    return this.request<T>({ ...config, method: 'DELETE', url });
  }
}

export class HetumindError extends Error {
  constructor(
    message: string,
    public status?: number,
    public code?: string,
    public details?: Record<string, any>
  ) {
    super(message);
    this.name = 'HetumindError';
  }
}

export default HetumindClient;