import { HetuflowClient } from "../utils/client.js";
import { GenerateTokenRequest, GenerateTokenResponse } from "../types/index.js";

export class AuthAPI {
  constructor(private client: HetuflowClient) {}

  /**
   * 生成 JWE Token (仅本机访问)
   */
  async generateToken(request: GenerateTokenRequest): Promise<GenerateTokenResponse> {
    return this.client.post<GenerateTokenResponse>("/api/v1/auth/generate-token", request);
  }
}
