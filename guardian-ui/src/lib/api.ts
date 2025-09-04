import type { ApiResponse } from './types';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

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

    try {
      const response = await fetch(url, {
        ...options,
        headers: {
          ...defaultHeaders,
          ...options.headers,
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const result = await response.json();
      
      // Handle the new API response format
      if (result.success) {
        return { ok: true, data: result.data };
      } else {
        return {
          ok: false,
          error: result.error || 'Unknown error',
        };
      }
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // Server endpoints
  async getServers() {
    return this.request('/servers');
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
    return this.request('/servers', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getServer(id: string) {
    return this.request(`/servers/${id}`);
  }

  async getServerHealth(id: string) {
    return this.request(`/servers/${id}/health`);
  }

  async startServer(id: string) {
    return this.request(`/servers/${id}/start`, {
      method: 'POST',
    });
  }

  async stopServer(id: string) {
    return this.request(`/servers/${id}/stop`, {
      method: 'POST',
    });
  }

  async restartServer(id: string) {
    return this.request(`/servers/${id}/restart`, {
      method: 'POST',
    });
  }

  async sendServerCommand(id: string, command: string) {
    return this.request(`/servers/${id}/command`, {
      method: 'POST',
      body: JSON.stringify({ command }),
    });
  }

  // Console endpoints
  async getConsoleMessages(id: string) {
    return this.request(`/servers/${id}/console`);
  }

  async sendConsoleCommand(id: string, cmd: string) {
    return this.request(`/servers/${id}/console`, {
      method: 'POST',
      body: JSON.stringify({ command: cmd }),
    });
  }

  // Player endpoints
  async getPlayers(id: string) {
    return this.request(`/servers/${id}/players`);
  }

  async getPlayer(id: string, uuid: string) {
    return this.request(`/servers/${id}/players/${uuid}`);
  }

  async kickPlayer(id: string, uuid: string) {
    return this.request(`/servers/${id}/players/${uuid}/kick`, {
      method: 'POST',
    });
  }

  async banPlayer(id: string, uuid: string) {
    return this.request(`/servers/${id}/players/${uuid}/ban`, {
      method: 'POST',
    });
  }

  async playerAction(
    id: string,
    uuid: string,
    action: 'message' | 'teleport' | 'kick' | 'ban' | 'throttle',
    data?: any
  ) {
    return this.request(`/servers/${id}/players/${uuid}/actions/${action}`, {
      method: 'POST',
      body: JSON.stringify(data || {}),
    });
  }

  // World endpoints
  async getWorldHeatmap(id: string) {
    return this.request(`/servers/${id}/world/heatmap`);
  }

  async getWorldFreezes(id: string) {
    return this.request(`/servers/${id}/world/freezes`);
  }

  async thawFreeze(id: string, actorId: string) {
    return this.request(`/servers/${id}/thaw/${actorId}`, {
      method: 'POST',
    });
  }

  async getChunkDetails(id: string, x: number, z: number) {
    return this.request(`/servers/${id}/world/chunks/${x}/${z}`);
  }

  // Mods & Rules endpoints
  async getMods(id: string) {
    return this.request(`/servers/${id}/mods`);
  }

  async getConflicts(id: string) {
    return this.request(`/servers/${id}/compat/conflicts`);
  }

  async getRules(id: string) {
    return this.request(`/servers/${id}/rules`);
  }

  async createRule(id: string, rule: any) {
    return this.request(`/servers/${id}/rules`, {
      method: 'POST',
      body: JSON.stringify(rule),
    });
  }

  async promoteRule(id: string, ruleId: string) {
    return this.request(`/servers/${id}/rules/promote`, {
      method: 'POST',
      body: JSON.stringify({ ruleId }),
    });
  }

  async runRuleLab(id: string, ruleDraft: string, target: string) {
    return this.request(`/servers/${id}/lab/run`, {
      method: 'POST',
      body: JSON.stringify({ ruleDraft, target }),
    });
  }

  // Performance endpoints
  async getMetrics(id: string, range: string = '1h') {
    return this.request(`/servers/${id}/metrics?range=${range}`);
  }

  async updateBudgets(id: string, budgets: {
    entityMs: number;
    tileMs: number;
    worldMs: number;
  }) {
    return this.request(`/servers/${id}/perf/budgets`, {
      method: 'POST',
      body: JSON.stringify(budgets),
    });
  }

  // Backup endpoints
  async getBackups(id: string) {
    return this.request(`/servers/${id}/backups`);
  }

  async createBackup(id: string) {
    return this.request(`/servers/${id}/backups`, {
      method: 'POST',
    });
  }

  async getBackup(id: string, backupId: string) {
    return this.request(`/servers/${id}/backups/${backupId}`);
  }

  async restoreBackup(id: string, backupId: string) {
    return this.request(`/servers/${id}/backups/${backupId}/restore`, {
      method: 'POST',
    });
  }

  async deleteBackup(id: string, backupId: string) {
    return this.request(`/servers/${id}/backups/${backupId}`, {
      method: 'DELETE',
    });
  }

  // Event endpoints
  async getEvents(id: string) {
    return this.request(`/servers/${id}/events`);
  }

  async createEvent(id: string, event: any) {
    return this.request(`/servers/${id}/events`, {
      method: 'POST',
      body: JSON.stringify(event),
    });
  }

  // Pregen endpoints
  async getPregenJobs(id: string) {
    return this.request(`/servers/${id}/pregen`);
  }

  async createPregenJob(id: string, data: {
    region: { x: number; z: number; radius: number };
    dimension: string;
    priority: 'low' | 'normal' | 'high';
  }) {
    return this.request(`/servers/${id}/pregen`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getPregenJob(id: string, jobId: string) {
    return this.request(`/servers/${id}/pregen/${jobId}`);
  }

  async updatePregenJob(id: string, jobId: string, data: any) {
    return this.request(`/servers/${id}/pregen/${jobId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deletePregenJob(id: string, jobId: string) {
    return this.request(`/servers/${id}/pregen/${jobId}`, {
      method: 'DELETE',
    });
  }

  async startPregenJob(id: string, jobId: string) {
    return this.request(`/servers/${id}/pregen/${jobId}/start`, {
      method: 'POST',
    });
  }

  async stopPregenJob(id: string, jobId: string) {
    return this.request(`/servers/${id}/pregen/${jobId}/stop`, {
      method: 'POST',
    });
  }

  // Sharding endpoints
  async getShardingTopology() {
    return this.request('/sharding/topology');
  }

  async updateShardingAssignments(assignments: any) {
    return this.request('/sharding/assignments', {
      method: 'POST',
      body: JSON.stringify({ assignments }),
    });
  }

  // Diagnostics endpoints
  async getDiagnostics(id: string) {
    return this.request(`/servers/${id}/diagnostics`);
  }

  async createDiagnosticBundle(id: string) {
    return this.request(`/servers/${id}/diagnostics/bundle`, {
      method: 'POST',
    });
  }

  // Settings endpoints
  async getServerSettings(id: string) {
    return this.request(`/servers/${id}/settings`);
  }

  async updateServerSettings(id: string, settings: any) {
    return this.request(`/servers/${id}/settings`, {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }

  // Workspace endpoints
  async getWorkspaceSettings() {
    return this.request('/workspace/settings');
  }

  async updateWorkspaceSettings(settings: any) {
    return this.request('/workspace/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }
}

export const api = new ApiClient();
export default api;
