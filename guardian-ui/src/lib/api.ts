import type { ApiResponse } from './types';
import config from './config';

const API_BASE_URL = config.api.baseUrl;

class ApiClient {
  private baseUrl: string;
  private retryCount: number = 0;
  private maxRetries: number = 3;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async retryRequest<T>(
    endpoint: string,
    options: RequestInit = {},
    retryCount: number = 0
  ): Promise<ApiResponse<T>> {
    const result = await this.request<T>(endpoint, options);
    
    // If request failed and we haven't exceeded max retries, try again
    if (!result.ok && retryCount < this.maxRetries) {
      console.log(`Retrying request (${retryCount + 1}/${this.maxRetries}): ${endpoint}`);
      await new Promise(resolve => setTimeout(resolve, 1000 * (retryCount + 1))); // Exponential backoff
      return this.retryRequest<T>(endpoint, options, retryCount + 1);
    }
    
    return result;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    
    const defaultHeaders = {
      'Content-Type': 'application/json',
    };

    console.log(`API Request: ${options.method || 'GET'} ${url}`);
    console.log('Request body:', options.body);

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 10000); // 10 second timeout

      const response = await fetch(url, {
        ...options,
        headers: {
          ...defaultHeaders,
          ...options.headers,
        },
        signal: controller.signal,
      });

      clearTimeout(timeoutId);
      console.log(`API Response: ${response.status} ${response.statusText}`);

      if (!response.ok) {
        // Handle specific HTTP status codes
        if (response.status === 404) {
          return {
            ok: false,
            error: 'Endpoint not found - server may not be running',
            data: undefined
          };
        } else if (response.status === 500) {
          return {
            ok: false,
            error: 'Server error - please check backend logs',
            data: undefined
          };
        } else if (response.status === 0) {
          return {
            ok: false,
            error: 'Cannot connect to server - is the backend running?',
            data: undefined
          };
        }
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      console.log('API Response data:', result);
      
      // Handle the API response format from the real backend
      if (result.success) {
        return { ok: true, data: result.data };
      } else {
        console.error('API Error:', result.error);
        return {
          ok: false,
          error: result.error || 'Unknown error occurred',
          data: undefined
        };
      }
    } catch (error) {
      console.error('API Request failed:', error);
      
      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          return {
            ok: false,
            error: 'Request timeout - server may be slow or unresponsive',
            data: undefined
          };
        } else if (error.message.includes('Failed to fetch')) {
          return {
            ok: false,
            error: 'Cannot connect to server - is the backend running on localhost:8080?',
            data: undefined
          };
        }
      }
      
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Network error',
        data: undefined
      };
    }
  }

  // Server endpoints
  async getServers() {
    return this.retryRequest('/api/servers');
  }

  async createServer(data: {
    name: string;
    loader: string;
    version: string;
    maxPlayers?: number;
    memory?: number;
    paths: {
      world: string;
      mods: string;
      config: string;
    };
    jarPath?: string;
  }) {
    return this.request('/api/servers', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getServer(id: string) {
    return this.request(`/api/servers/${id}`);
  }

  async getServerHealth(id: string) {
    return this.request(`/api/servers/${id}/health`);
  }

  async startServer(id: string) {
    return this.request(`/api/servers/${id}/start`, {
      method: 'POST',
    });
  }

  async stopServer(id: string) {
    return this.request(`/api/servers/${id}/stop`, {
      method: 'POST',
    });
  }

  async restartServer(id: string) {
    return this.request(`/api/servers/${id}/restart`, {
      method: 'POST',
    });
  }

  async sendServerCommand(id: string, command: string) {
    return this.request(`/api/servers/${id}/command`, {
      method: 'POST',
      body: JSON.stringify({ command }),
    });
  }

  // EULA endpoints
  async getEulaStatus(id: string) {
    return this.request(`/api/servers/${id}/eula`);
  }

  async acceptEula(id: string) {
    return this.request(`/api/servers/${id}/eula/accept`, {
      method: 'POST',
    });
  }

  // Server.properties endpoints (RCON toggle, etc.)
  async getServerPropertiesConfig(id: string) {
    return this.request(`/api/servers/${id}/config/server.properties`);
  }

  async updateServerPropertiesConfig(id: string, properties: Record<string, string>) {
    return this.request(`/api/servers/${id}/config/server.properties`, {
      method: 'PUT',
      body: JSON.stringify(properties),
    });
  }

  async deleteServer(id: string) {
    return this.request(`/api/servers/${id}`, {
      method: 'DELETE',
    });
  }

  // Console endpoints
  async getConsoleMessages(id: string) {
    return this.request(`/api/servers/${id}/console`);
  }

  async sendConsoleCommand(id: string, cmd: string) {
    return this.request(`/api/servers/${id}/console`, {
      method: 'POST',
      body: JSON.stringify({ command: cmd }),
    });
  }

  // Player endpoints
  async getPlayers(id: string) {
    return this.request(`/api/servers/${id}/players`);
  }

  async getPlayer(id: string, uuid: string) {
    return this.request(`/api/servers/${id}/players/${uuid}`);
  }

  async kickPlayer(id: string, uuid: string) {
    return this.request(`/api/servers/${id}/players/${uuid}/kick`, {
      method: 'POST',
    });
  }

  async banPlayer(id: string, uuid: string) {
    return this.request(`/api/servers/${id}/players/${uuid}/ban`, {
      method: 'POST',
    });
  }

  async playerAction(
    id: string,
    uuid: string,
    action: 'message' | 'teleport' | 'kick' | 'ban' | 'throttle',
    data?: any
  ) {
    return this.request(`/api/servers/${id}/players/${uuid}/actions/${action}`, {
      method: 'POST',
      body: JSON.stringify(data || {}),
    });
  }

  // World endpoints
  async getWorldHeatmap(id: string) {
    return this.request(`/api/servers/${id}/world/heatmap`);
  }

  async getWorldFreezes(id: string) {
    return this.request(`/api/servers/${id}/world/freezes`);
  }

  async thawFreeze(id: string, actorId: string) {
    return this.request(`/api/servers/${id}/thaw/${actorId}`, {
      method: 'POST',
    });
  }

  async getChunkDetails(id: string, x: number, z: number) {
    return this.request(`/api/servers/${id}/world/chunks/${x}/${z}`);
  }

  // Mods & Rules endpoints
  async getMods(id: string) {
    return this.request(`/api/servers/${id}/mods`);
  }

  async getConflicts(id: string) {
    return this.request(`/api/servers/${id}/compat/conflicts`);
  }

  async getRules(id: string) {
    return this.request(`/api/servers/${id}/rules`);
  }

  async createRule(id: string, rule: any) {
    return this.request(`/api/servers/${id}/rules`, {
      method: 'POST',
      body: JSON.stringify(rule),
    });
  }

  // Modpack & Mod Search endpoints
  async searchMods(filters: any) {
    return this.request(`/api/v1/mods/search`, {
      method: 'POST',
      body: JSON.stringify(filters),
    });
  }

  async getMod(id: string) {
    return this.request(`/api/v1/mods/${id}`);
  }

  async getModpacks() {
    return this.request(`/api/v1/modpacks`);
  }

  async getModpack(id: string) {
    return this.request(`/api/v1/modpacks/${id}`);
  }

  async createModpack(modpack: any) {
    return this.request(`/api/v1/modpacks`, {
      method: 'POST',
      body: JSON.stringify(modpack),
    });
  }

  async updateModpack(id: string, modpack: any) {
    return this.request(`/api/v1/modpacks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(modpack),
    });
  }

  async deleteModpack(id: string) {
    return this.request(`/api/v1/modpacks/${id}`, {
      method: 'DELETE',
    });
  }

  async getMinecraftVersions() {
    return this.request(`/api/v1/minecraft/versions`);
  }

  async checkModpackCompatibility(modpackId: string, modpack: any) {
    return this.request(`/api/v1/modpacks/${modpackId}/compatibility`, {
      method: 'POST',
      body: JSON.stringify(modpack),
    });
  }

  // Server mod management endpoints
  async addModToServer(serverId: string, modId: string, version: string) {
    return this.request(`/api/servers/${serverId}/mods`, {
      method: 'POST',
      body: JSON.stringify({ modId, version }),
    });
  }

  async removeModFromServer(serverId: string, modId: string) {
    return this.request(`/api/servers/${serverId}/mods/${modId}`, {
      method: 'DELETE',
    });
  }

  async applyModpackToServer(serverId: string, modpackId: string) {
    return this.request(`/api/servers/${serverId}/modpacks/apply`, {
      method: 'POST',
      body: JSON.stringify({ modpackId }),
    });
  }

  async promoteRule(id: string, ruleId: string) {
    return this.request(`/api/servers/${id}/rules/promote`, {
      method: 'POST',
      body: JSON.stringify({ ruleId }),
    });
  }

  async runRuleLab(id: string, ruleDraft: string, target: string) {
    return this.request(`/api/servers/${id}/lab/run`, {
      method: 'POST',
      body: JSON.stringify({ ruleDraft, target }),
    });
  }

  // Performance endpoints
  async getMetrics(id: string, range: string = '1h') {
    return this.request(`/api/servers/${id}/metrics?range=${range}`);
  }

  async getRealtimeMetrics(id: string) {
    return this.request(`/api/servers/${id}/metrics/realtime`);
  }

  async updateBudgets(id: string, budgets: {
    entityMs: number;
    tileMs: number;
    worldMs: number;
  }) {
    return this.request(`/api/servers/${id}/perf/budgets`, {
      method: 'POST',
      body: JSON.stringify(budgets),
    });
  }

  // Backup endpoints
  async getBackups(id: string) {
    return this.request(`/api/servers/${id}/backups`);
  }

  async createBackup(id: string) {
    return this.request(`/api/servers/${id}/backups`, {
      method: 'POST',
    });
  }

  async getBackup(id: string, backupId: string) {
    return this.request(`/api/servers/${id}/backups/${backupId}`);
  }

  async restoreBackup(id: string, backupId: string) {
    return this.request(`/api/servers/${id}/backups/${backupId}/restore`, {
      method: 'POST',
    });
  }

  async deleteBackup(id: string, backupId: string) {
    return this.request(`/api/servers/${id}/backups/${backupId}`, {
      method: 'DELETE',
    });
  }

  // Event endpoints
  async getEvents(id: string) {
    return this.request(`/api/servers/${id}/events`);
  }

  async createEvent(id: string, event: any) {
    return this.request(`/api/servers/${id}/events`, {
      method: 'POST',
      body: JSON.stringify(event),
    });
  }

  // World data endpoints
  async getWorldData(id: string) {
    return this.request(`/api/servers/${id}/world`);
  }

  async getFreezes(id: string) {
    return this.request(`/api/servers/${id}/world/freezes`);
  }

  // Pregen endpoints
  async getPregenJobs(id: string) {
    return this.request(`/api/servers/${id}/pregen`);
  }

  async createPregenJob(id: string, data: {
    region: { x: number; z: number; radius: number };
    dimension: string;
    priority: 'low' | 'normal' | 'high';
  }) {
    return this.request(`/api/servers/${id}/pregen`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getPregenJob(id: string, jobId: string) {
    return this.request(`/api/servers/${id}/pregen/${jobId}`);
  }

  async updatePregenJob(id: string, jobId: string, data: any) {
    return this.request(`/api/servers/${id}/pregen/${jobId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deletePregenJob(id: string, jobId: string) {
    return this.request(`/api/servers/${id}/pregen/${jobId}`, {
      method: 'DELETE',
    });
  }

  async startPregenJob(id: string, jobId: string) {
    return this.request(`/api/servers/${id}/pregen/${jobId}/start`, {
      method: 'POST',
    });
  }

  async stopPregenJob(id: string, jobId: string) {
    return this.request(`/api/servers/${id}/pregen/${jobId}/stop`, {
      method: 'POST',
    });
  }

  // Sharding endpoints
  async getShardingTopology() {
    return this.request('/api/sharding/topology');
  }

  async getShardingAssignments() {
    return this.request('/api/sharding/assignments');
  }

  async updateShardingAssignments(assignments: any) {
    return this.request('/api/sharding/assignments', {
      method: 'POST',
      body: JSON.stringify({ assignments }),
    });
  }

  // Diagnostics endpoints
  async getDiagnostics(id: string) {
    return this.request(`/api/servers/${id}/diagnostics`);
  }

  async createDiagnosticBundle(id: string) {
    return this.request(`/api/servers/${id}/diagnostics/bundle`, {
      method: 'POST',
    });
  }

  // Settings endpoints
  async getServerSettings(id: string) {
    return this.request(`/api/servers/${id}/settings`);
  }

  async updateServerSettings(id: string, settings: any) {
    return this.request(`/api/servers/${id}/settings`, {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }

  // Token management
  async createToken(id: string, tokenData: any) {
    return this.request(`/api/servers/${id}/tokens`, {
      method: 'POST',
      body: JSON.stringify(tokenData),
    });
  }

  async updateToken(id: string, tokenId: string, data: any) {
    return this.request(`/api/servers/${id}/tokens/${tokenId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deleteToken(id: string, tokenId: string) {
    return this.request(`/api/servers/${id}/tokens/${tokenId}`, {
      method: 'DELETE',
    });
  }

  // Server configuration file endpoints
  async getServerConfig(id: string) {
    return this.request(`/api/servers/${id}/config`);
  }

  async updateServerConfig(id: string, config: any) {
    return this.request(`/api/servers/${id}/config`, {
      method: 'PUT',
      body: JSON.stringify(config),
    });
  }

  async getServerProperties(id: string) {
    return this.request(`/api/servers/${id}/config/server.properties`);
  }

  async updateServerProperties(id: string, properties: Record<string, string>) {
    return this.request(`/api/servers/${id}/config/server.properties`, {
      method: 'PUT',
      body: JSON.stringify(properties),
    });
  }

  async getServerJVMArgs(id: string) {
    return this.request(`/api/servers/${id}/config/jvm-args`);
  }

  async updateServerJVMArgs(id: string, args: string[]) {
    return this.request(`/api/servers/${id}/config/jvm-args`, {
      method: 'PUT',
      body: JSON.stringify({ args }),
    });
  }

  // Workspace endpoints
  async getWorkspaceSettings() {
    return this.request('/api/workspace/settings');
  }

  async updateWorkspaceSettings(settings: any) {
    return this.request('/api/workspace/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }

  // Health check
  async healthCheck() {
    return this.request('/api/health');
  }

  async getStatus() {
    return this.request('/api/status');
  }
}

export const api = new ApiClient();
export default api;