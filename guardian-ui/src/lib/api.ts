const TRY_PORTS = Array.from({length: 13}, (_,i)=>8080+i);

let cachedBase: string | null = null;
async function ping(base: string) {
  try { 
    const r = await fetch(`${base}/healthz`, { cache: "no-store" }); 
    return r.ok; 
  } catch { 
    return false; 
  }
}

export async function getAPI_BASE(): Promise<string> {
  if (cachedBase) return cachedBase;
  // env override
  const env = import.meta.env.VITE_API_BASE?.replace(/\/+$/, "");
  if (env && await ping(env)) { cachedBase = env; return env; }

  // discover localhost ports
  for (const p of TRY_PORTS) {
    const base = `http://127.0.0.1:${p}`;
    if (await ping(base)) { cachedBase = base; return cachedBase; }
  }
  // last fallback: read port file through Tauri (optional)
  cachedBase = "http://127.0.0.1:8080";
  return cachedBase;
}

// For backward compatibility, we'll use a default that gets updated
export let API_BASE = "http://127.0.0.1:8080";

// Initialize the API base
getAPI_BASE().then(base => {
  API_BASE = base;
});

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const base = await getAPI_BASE();
  const res = await fetch(`${base}${path}`, {
    headers: { "Content-Type": "application/json", ...(init?.headers||{}) },
    ...init,
  });
  if (!res.ok) {
    let err: any = { status: res.status, message: res.statusText };
    try { err = await res.json(); } catch {}
    throw err;
  }
  if (res.status === 204) return undefined as T;
  return res.json() as Promise<T>;
}

// Internal API function
async function apiCall<T>(path: string, init?: RequestInit): Promise<T> {
  return api<T>(path, init);
}

// Extended API client with specific methods for backward compatibility
export const apiClient = {
  // Generic method
  async call<T>(path: string, init?: RequestInit): Promise<T> {
    return apiCall<T>(path, init);
  },

  // Server management
  async getServers(): Promise<any> {
    return apiCall('/api/servers');
  },
  async getServer(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}`);
  },
  async createServer(data: any): Promise<any> {
    return apiCall('/api/servers', { method: 'POST', body: JSON.stringify(data) });
  },
  async deleteServer(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}`, { method: 'DELETE' });
  },

  // Server control
  async startServer(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/start`, { method: 'POST' });
  },
  async stopServer(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/stop`, { method: 'POST' });
  },
  async restartServer(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/restart`, { method: 'POST' });
  },

  // Server health and status
  async getServerHealth(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/health`);
  },
  async getPlayers(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/players`);
  },
  async getRealtimeMetrics(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/metrics`);
  },
  async getMetrics(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/metrics`);
  },

  // Console and commands
  async getConsoleMessages(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/console`);
  },
  async sendConsoleCommand(serverId: string, command: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/console`, { 
      method: 'POST', 
      body: JSON.stringify({ command }) 
    });
  },
  async sendServerCommand(serverId: string, command: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/command`, { 
      method: 'POST', 
      body: JSON.stringify({ command }) 
    });
  },

  // EULA
  async getEulaStatus(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/eula`);
  },
  async acceptEula(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/eula`, { method: 'POST' });
  },

  // Backups
  async getBackups(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/backups`);
  },
  async createBackup(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/backups`, { method: 'POST' });
  },
  async deleteBackup(serverId: string, snapshotId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/backups/${snapshotId}`, { method: 'DELETE' });
  },

  // Diagnostics
  async getDiagnostics(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/diagnostics`);
  },

  // Events
  async getEvents(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/events`);
  },
  async createEvent(serverId: string, eventData: any): Promise<any> {
    return apiCall(`/api/servers/${serverId}/events`, { 
      method: 'POST', 
      body: JSON.stringify(eventData) 
    });
  },

  // Mods and rules
  async getMods(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/mods`);
  },
  async getRules(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/rules`);
  },
  async getConflicts(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/conflicts`);
  },

  // Settings
  async getServerSettings(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/settings`);
  },
  async updateServerSettings(serverId: string, settings: any): Promise<any> {
    return apiCall(`/api/servers/${serverId}/settings`, { 
      method: 'PUT', 
      body: JSON.stringify(settings) 
    });
  },
  async getServerConfig(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/config`);
  },
  async updateServerConfig(serverId: string, config: any): Promise<any> {
    return apiCall(`/api/servers/${serverId}/config`, { 
      method: 'PUT', 
      body: JSON.stringify(config) 
    });
  },
  async getServerProperties(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/properties`);
  },
  async updateServerProperties(serverId: string, properties: any): Promise<any> {
    return apiCall(`/api/servers/${serverId}/properties`, { 
      method: 'PUT', 
      body: JSON.stringify(properties) 
    });
  },
  async getServerJVMArgs(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/jvm-args`);
  },
  async updateServerJVMArgs(serverId: string, args: any): Promise<any> {
    return apiCall(`/api/servers/${serverId}/jvm-args`, { 
      method: 'PUT', 
      body: JSON.stringify(args) 
    });
  },

  // Sharding
  async getShardingTopology(): Promise<any> {
    return apiCall('/api/sharding/topology');
  },
  async getShardingAssignments(): Promise<any> {
    return apiCall('/api/sharding/assignments');
  },

  // Player actions
  async playerAction(serverId: string, playerUuid: string, action: string, data?: any): Promise<any> {
    return apiCall(`/api/servers/${serverId}/players/${playerUuid}/${action}`, { 
      method: 'POST', 
      body: JSON.stringify(data || {}) 
    });
  },
  async kickPlayer(serverId: string, playerUuid: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/players/${playerUuid}/kick`, { method: 'POST' });
  },
  async banPlayer(serverId: string, playerUuid: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/players/${playerUuid}/ban`, { method: 'POST' });
  },

  // Pregen jobs
  async getPregenJobs(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/pregen`);
  },
  async createPregenJob(serverId: string, data: any): Promise<any> {
    return apiCall(`/api/servers/${serverId}/pregen`, { 
      method: 'POST', 
      body: JSON.stringify(data) 
    });
  },
  async startPregenJob(serverId: string, jobId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/pregen/${jobId}/start`, { method: 'POST' });
  },
  async stopPregenJob(serverId: string, jobId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/pregen/${jobId}/stop`, { method: 'POST' });
  },
  async deletePregenJob(serverId: string, jobId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/pregen/${jobId}`, { method: 'DELETE' });
  },

  // World
  async getWorldFreezes(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/world/freezes`);
  },

  // Tokens
  async createToken(serverId: string, data: any): Promise<any> {
    return apiCall(`/api/servers/${serverId}/tokens`, { 
      method: 'POST', 
      body: JSON.stringify(data) 
    });
  },
  async deleteToken(serverId: string, tokenId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/tokens/${tokenId}`, { method: 'DELETE' });
  },
  async updateToken(serverId: string, tokenId: string, data: any): Promise<any> {
    return apiCall(`/api/servers/${serverId}/tokens/${tokenId}`, { 
      method: 'PUT', 
      body: JSON.stringify(data) 
    });
  },
};

// Export the apiClient as default export for backward compatibility
export default apiClient;
