import { PageResult } from '@fusion-data/fusionsql';
import { HetumindClient } from '../utils/client.js';
import {
  CredentialForInsert,
  CredentialForUpdate,
  CredentialForQuery,
  CredentialEntity,
  CredentialWithDecryptedData,
  VerifyCredentialRequest,
  CredentialVerifyResult,
  CredentialReferencesResponse,
  IdUuidResult,
} from '../types/index.js';

export class CredentialAPI {
  constructor(private client: HetumindClient) {}

  /**
   * Create a new credential
   */
  async createCredential(data: CredentialForInsert): Promise<IdUuidResult> {
    return this.client.post<IdUuidResult>('/api/v1/credentials', data);
  }

  /**
   * Query credentials with filters
   */
  async queryCredentials(query: CredentialForQuery): Promise<PageResult<CredentialEntity>> {
    return this.client.post<PageResult<CredentialEntity>>('/api/v1/credentials/query', query);
  }

  /**
   * Get credential details by ID (with decrypted data)
   */
  async getCredential(id: string): Promise<CredentialWithDecryptedData> {
    return this.client.get<CredentialWithDecryptedData>(`/api/v1/credentials/${id}`);
  }

  /**
   * Update a credential
   */
  async updateCredential(id: string, data: CredentialForUpdate): Promise<void> {
    return this.client.put<void>(`/api/v1/credentials/${id}`, data);
  }

  /**
   * Delete a credential
   */
  async deleteCredential(id: string): Promise<void> {
    return this.client.delete<void>(`/api/v1/credentials/${id}`);
  }

  /**
   * Verify a credential (without saving)
   */
  async verifyCredential(request: VerifyCredentialRequest): Promise<CredentialVerifyResult> {
    return this.client.post<CredentialVerifyResult>('/api/v1/credentials/verify', request);
  }

  /**
   * Verify a stored credential
   */
  async verifyStoredCredential(id: string): Promise<CredentialVerifyResult> {
    return this.client.post<CredentialVerifyResult>(`/api/v1/credentials/${id}/verify`);
  }

  /**
   * Get credential references
   */
  async getCredentialReferences(id: string): Promise<CredentialReferencesResponse> {
    return this.client.get<CredentialReferencesResponse>(`/api/v1/credentials/${id}/references`);
  }
}
