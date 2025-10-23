import { PageResult } from '@fusion-data/fusionsql';
import { HetumindClient } from '../utils/client.js';
import {
  WorkflowForCreate,
  WorkflowForUpdate,
  WorkflowForQuery,
  Workflow,
  ValidateWorkflowRequest,
  ValidateWorkflowResponse,
  ExecuteWorkflowRequest,
  ExecutionIdResponse,
  IdUuidResult,
} from '../types/index.js';

export class WorkflowAPI {
  constructor(private client: HetumindClient) {}

  /**
   * Create a new workflow
   */
  async createWorkflow(data: WorkflowForCreate): Promise<IdUuidResult> {
    return this.client.post<IdUuidResult>('/api/v1/workflows', data);
  }

  /**
   * Query workflows with filters
   */
  async queryWorkflows(query: WorkflowForQuery): Promise<PageResult<Workflow>> {
    return this.client.post<PageResult<Workflow>>('/api/v1/workflows/query', query);
  }

  /**
   * Validate a workflow definition
   */
  async validateWorkflow(request: ValidateWorkflowRequest): Promise<ValidateWorkflowResponse> {
    return this.client.post<ValidateWorkflowResponse>('/api/v1/workflows/validate', request);
  }

  /**
   * Get a workflow by ID
   */
  async getWorkflow(id: string): Promise<Workflow> {
    return this.client.get<Workflow>(`/api/v1/workflows/${id}`);
  }

  /**
   * Update a workflow
   */
  async updateWorkflow(id: string, data: WorkflowForUpdate): Promise<IdUuidResult> {
    return this.client.put<IdUuidResult>(`/api/v1/workflows/${id}`, data);
  }

  /**
   * Delete a workflow
   */
  async deleteWorkflow(id: string): Promise<void> {
    return this.client.delete<void>(`/api/v1/workflows/${id}`);
  }

  /**
   * Execute a workflow
   */
  async executeWorkflow(id: string, request: ExecuteWorkflowRequest): Promise<ExecutionIdResponse> {
    return this.client.post<ExecutionIdResponse>(`/api/v1/workflows/${id}/execute`, request);
  }

  /**
   * Activate a workflow
   */
  async activateWorkflow(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/workflows/${id}/activate`);
  }

  /**
   * Deactivate a workflow
   */
  async deactivateWorkflow(id: string): Promise<void> {
    return this.client.post<void>(`/api/v1/workflows/${id}/deactivate`);
  }

  /**
   * Duplicate a workflow
   */
  async duplicateWorkflow(id: string): Promise<IdUuidResult> {
    return this.client.post<IdUuidResult>(`/api/v1/workflows/${id}/duplicate`);
  }
}
