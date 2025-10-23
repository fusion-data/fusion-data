import { HetumindClient, HetumindClientConfig } from './utils/client.js';
import { WorkflowAPI } from './api/workflows.js';
import { ExecutionAPI } from './api/executions.js';
import { CredentialAPI } from './api/credentials.js';
import { UserAPI } from './api/users.js';

export class HetumindSDK {
  private client: HetumindClient;

  public readonly workflows: WorkflowAPI;
  public readonly executions: ExecutionAPI;
  public readonly credentials: CredentialAPI;
  public readonly users: UserAPI;

  constructor(config: HetumindClientConfig) {
    this.client = new HetumindClient(config);

    // Initialize API modules
    this.workflows = new WorkflowAPI(this.client);
    this.executions = new ExecutionAPI(this.client);
    this.credentials = new CredentialAPI(this.client);
    this.users = new UserAPI(this.client);
  }

  /**
   * Set authentication token
   */
  setToken(token: string): void {
    this.client.setToken(token);
  }

  /**
   * Clear authentication token
   */
  clearToken(): void {
    this.client.clearToken();
  }

  /**
   * Create a new SDK instance
   */
  static create(config: HetumindClientConfig): HetumindSDK {
    return new HetumindSDK(config);
  }
}

// Default export
export default HetumindSDK;