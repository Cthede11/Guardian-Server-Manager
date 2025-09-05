import type { ApiResponse } from './types';
import config from './config';

const API_BASE_URL = config.api.baseUrl;

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
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
      const response = await fetch(url, {
        ...options,
        headers: {
          ...defaultHeaders,
          ...options.headers,
        },
      });

      console.log(`API Response: ${response.status} ${response.statusText}`);

      if (!response.ok) {
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
          error: result.error || 'Unknown error',
        };
      }
    } catch (error) {
      console.error('API Request failed:', error);
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // Server endpoints
  async getServers() {
    return this.request('/api/servers');
  }

  async createServer(data: {
    name: string;
    loader: string;
    version: string;
    paths: {
      world: string;
      mods: string;
      config: string;
    };
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