import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

export interface HetuflowClientConfig {
  baseURL: string;
  timeout?: number;
  headers?: Record<string, string>;
  token?: string;
}

export class HetuflowClient {
  private client: AxiosInstance;

  constructor(config: HetuflowClientConfig) {
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
      config => {
        // 添加认证 token
        if (this.token) {
          config.headers.Authorization = `Bearer ${this.token}`;
        }
        return config;
      },
      error => {
        return Promise.reject(error);
      }
    );

    // 添加响应拦截器
    this.client.interceptors.response.use(
      response => {
        return response;
      },
      error => {
        // 统一错误处理
        if (error.response?.data) {
          throw new HetuflowError(
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

  private token?: string;

  setToken(token: string) {
    this.token = token;
  }

  clearToken() {
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

export class HetuflowError extends Error {
  constructor(
    message: string,
    public status?: number,
    public code?: string,
    public details?: Record<string, any>
  ) {
    super(message);
    this.name = 'HetuflowError';
  }
}

export default HetuflowClient;
