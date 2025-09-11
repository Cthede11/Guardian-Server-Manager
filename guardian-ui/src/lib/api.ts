const COMMON = new Set([80,443,3000,5173,8000,8080,9000]);
const DEF_MIN = 52100, DEF_MAX = 52150;

function parseRange(): [number,number] {
  const env = import.meta.env.VITE_HOSTD_PORT_RANGE as string | undefined;
  if (env && /^\d{2,5}-\d{2,5}$/.test(env)) {
    const [a,b]=env.split('-').map(n=>parseInt(n,10)); if(a<b) return [a,b];
  }
  return [DEF_MIN, DEF_MAX];
}
function* candidatePorts(): Generator<number> {
  const [min,max]=parseRange();
  for (let p=min; p<=max; p++) if(!COMMON.has(p)) yield p;
}

let cachedBase: string | null = null;

async function ping(base: string) {
  try { const r = await fetch(`${base}/healthz`, { cache: "no-store" }); return r.ok; } catch { return false; }
}

// Function to update the API base URL (called from Tauri command)
export async function updateApiBase(newBase: string) {
  cachedBase = newBase;
  API_BASE = newBase;
  console.log('API base URL updated to:', newBase);
}

export async function getAPI_BASE(): Promise<string> {
  if (cachedBase) return cachedBase;
  const env = (import.meta.env.VITE_API_BASE || "").replace(/\/+$/,"");
  if (env && await ping(env)) return (cachedBase = env);

  for (const p of candidatePorts()) {
    const base = `http://127.0.0.1:${p}`;
    if (await ping(base)) return (cachedBase = base);
  }
  // last fallback – try what backend wrote last time
  try {
    // If using Tauri, you can expose a readTextFile to get backend_port.txt; skip here if not available
  } catch {}
  // give up to default range start (frontend will still show banner if health fails)
  return (cachedBase = `http://127.0.0.1:${DEF_MIN}`);
}

export async function waitForBackend(timeoutMs = 15000) {
  const start = Date.now();
  let base = await getAPI_BASE();
  while (Date.now() - start < timeoutMs) {
    if (await ping(base)) return base;
    await new Promise(r=>setTimeout(r, 250));
    // re-scan in case backend moved
    cachedBase = null;
    base = await getAPI_BASE();
  }
  throw new Error("Backend not reachable");
}

// For backward compatibility, we'll use a default that gets updated
export let API_BASE = "http://127.0.0.1:8080";

// Initialize the API base
getAPI_BASE().then(base => {
  API_BASE = base;
});

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const base = await getAPI_BASE();
  const res = await fetch(`${base}${path}`, { headers: { "Content-Type":"application/json", ...(init?.headers||{}) }, ...init });
  if (!res.ok) {
    let err: any = { status: res.status, message: res.statusText };
    try { err = await res.json(); } catch {}
    throw err;
  }
  return res.status === 204 ? (undefined as T) : await res.json() as T;
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

  // Additional methods
  async getServerSummary(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}`);
  },
  async sendRcon(serverId: string, command: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/command`, { 
      method: 'POST', 
      body: JSON.stringify({ command }) 
    });
  },
  async promoteServer(serverId: string): Promise<any> {
    return apiCall(`/api/servers/${serverId}/promote`, { method: 'POST' });
  },
};

// Events for real-time communication
export const events = {
  // WebSocket events will be handled by the websocket module
  // These are placeholder implementations for now
  subscribeToConsole(serverId: string, callback: (data: any) => void): Promise<() => void> {
    // Placeholder implementation
    console.log(`Subscribing to console for server ${serverId}`);
    return Promise.resolve(() => console.log(`Unsubscribing from console for server ${serverId}`));
  },
  subscribeToMetrics(serverId: string, callback: (data: any) => void): Promise<() => void> {
    // Placeholder implementation
    console.log(`Subscribing to metrics for server ${serverId}`);
    return Promise.resolve(() => console.log(`Unsubscribing from metrics for server ${serverId}`));
  },
  subscribeToPlayers(serverId: string, callback: (data: any) => void): Promise<() => void> {
    // Placeholder implementation
    console.log(`Subscribing to players for server ${serverId}`);
    return Promise.resolve(() => console.log(`Unsubscribing from players for server ${serverId}`));
  },
  subscribeToFreezes(serverId: string, callback: (data: any) => void): Promise<() => void> {
    // Placeholder implementation
    console.log(`Subscribing to freezes for server ${serverId}`);
    return Promise.resolve(() => console.log(`Unsubscribing from freezes for server ${serverId}`));
  },
  subscribeToPregen(serverId: string, callback: (data: any) => void): Promise<() => void> {
    // Placeholder implementation
    console.log(`Subscribing to pregen for server ${serverId}`);
    return Promise.resolve(() => console.log(`Unsubscribing from pregen for server ${serverId}`));
  },
  subscribeToHealth(serverId: string, callback: (data: any) => void): Promise<() => void> {
    // Placeholder implementation
    console.log(`Subscribing to health for server ${serverId}`);
    return Promise.resolve(() => console.log(`Unsubscribing from health for server ${serverId}`));
  },
};

// Export the apiClient as default export for backward compatibility
export default apiClient;
