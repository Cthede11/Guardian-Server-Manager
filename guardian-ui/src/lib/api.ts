import type { ApiResponse } from './types';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000/api/v1';

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

      const data = await response.json();
      return { ok: true, data };
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

  async getServerSummary(id: string) {
    return this.request(`/servers/${id}/summary`);
  }

  async getServerHealth(id: string) {
    return this.request(`/servers/${id}/health`);
  }

  async serverAction(id: string, action: 'start' | 'stop' | 'restart' | 'promote') {
    return this.request(`/servers/${id}/actions/${action}`, {
      method: 'POST',
    });
  }

  // Console endpoints
  async sendConsoleCommand(id: string, cmd: string) {
    return this.request(`/servers/${id}/console/command`, {
      method: 'POST',
      body: JSON.stringify({ cmd }),
    });
  }

  // Player endpoints
  async getOnlinePlayers(id: string) {
    return this.request(`/servers/${id}/players/online`);
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
  async getWorldHeatmap(id: string, metric: string = 'tickCost') {
    return this.request(`/servers/${id}/world/heatmap?metric=${metric}`);
  }

  async getFreezes(id: string) {
    return this.request(`/servers/${id}/freezes`);
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
  async getSnapshots(id: string) {
    return this.request(`/servers/${id}/snapshots`);
  }

  async createSnapshot(id: string) {
    return this.request(`/servers/${id}/snapshots/create`, {
      method: 'POST',
    });
  }

  async restoreSnapshot(id: string, scope: string, params: any) {
    return this.request(`/servers/${id}/snapshots/restore`, {
      method: 'POST',
      body: JSON.stringify({ scope, params }),
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
  async startPregen(id: string, data: {
    region: { x: number; z: number; radius: number };
    dimension: string;
    priority: 'low' | 'normal' | 'high';
  }) {
    return this.request(`/servers/${id}/pregen`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getPregenStatus(id: string) {
    return this.request(`/servers/${id}/pregen/status`);
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
