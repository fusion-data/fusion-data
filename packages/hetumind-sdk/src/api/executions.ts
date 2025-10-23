import { PageResult } from '@fusion-data/fusionsql';
import { HetumindClient } from '../utils/client.js';
import {
  ExecutionForQuery,
  ExecutionResponse,
  ExecutionStatusResponse,
  ExecutionLogResponse,
} from '../types/index.js';

export class ExecutionAPI {
  constructor(private client: HetumindClient) {}

  /**
   * Query executions with filters
   */
  async queryExecutions(query: ExecutionForQuery): Promise<PageResult<ExecutionResponse>> {
    return this.client.post<PageResult<ExecutionResponse>>('/api/v1/executions/query', query);
  }

  /**
   * Get execution details by ID
   */
  async getExecution(id: string): Promise<ExecutionResponse> {
    return this.client.get<ExecutionResponse>(`/api/v1/executions/${id}`);
  }

  /**
   * Cancel an execution
   */
  async cancelExecution(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/executions/${id}/cancel`);
  }

  /**
   * Retry an execution
   */
  async retryExecution(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/executions/${id}/retry`);
  }

  /**
   * Get execution logs
   */
  async getExecutionLogs(id: string): Promise<ExecutionLogResponse> {
    return this.client.get<ExecutionLogResponse>(`/api/v1/executions/${id}/logs`);
  }

  /**
   * Get execution status (lightweight)
   */
  async getExecutionStatus(id: string): Promise<ExecutionStatusResponse> {
    return this.client.get<ExecutionStatusResponse>(`/api/v1/executions/${id}/status`);
  }

  /**
   * Stream execution logs (Server-Sent Events)
   * Note: This returns an EventSource for real-time updates
   */
  streamExecutionLogs(id: string): EventSource {
    const baseURL = this.client['client'].defaults.baseURL;
    const url = `${baseURL}/api/v1/executions/${id}/logs/stream`;

    // Note: EventSource doesn't support custom headers in all browsers
    // For production use, you might need to use a different approach
    // like WebSocket or a polling mechanism
    const eventSource = new EventSource(url);

    return eventSource;
  }
}
