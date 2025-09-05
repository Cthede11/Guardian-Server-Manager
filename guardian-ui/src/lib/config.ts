// Production configuration for Guardian
export const config = {
  // API Configuration
  api: {
    baseUrl: 'http://localhost:8080',
    timeout: 30000,
    retries: 3,
  },
  
  // WebSocket Configuration
  websocket: {
    url: 'ws://localhost:8080/ws',
    reconnectInterval: 5000,
    maxReconnectAttempts: 10,
  },
  
  // Application Configuration
  app: {
    name: 'Guardian',
    version: '1.0.0',
    environment: 'production',
  },
  
  // Server Configuration
  server: {
    defaultPort: 25565,
    maxServers: 10,
    defaultMemory: 2048,
  },
  
  // UI Configuration
  ui: {
    theme: 'dark',
    language: 'en',
    autoRefresh: true,
    refreshInterval: 30000,
  },
  
  // Logging Configuration
  logging: {
    level: 'info',
    enableConsole: true,
    enableFile: true,
  },
  
  // Security Configuration
  security: {
    enableAuth: true,
    sessionTimeout: 3600000, // 1 hour
    maxLoginAttempts: 5,
  },
  
  // Performance Configuration
  performance: {
    enableGpuAcceleration: true,
    maxConcurrentOperations: 5,
    cacheSize: 100,
  },
};

export default config;