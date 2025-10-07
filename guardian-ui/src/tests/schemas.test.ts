import { describe, it, expect } from 'vitest';
import { z } from 'zod';

// Import the schemas from the wizard components
const ServerFormDataSchema = z.object({
  // Basics
  name: z.string().min(1, 'Server name is required').max(50, 'Server name too long'),
  minecraftVersion: z.string().min(1, 'Minecraft version is required'),
  loader: z.enum(['vanilla', 'fabric', 'forge', 'quilt']),
  memory: z.number().min(512, 'Memory must be at least 512MB').max(32768, 'Memory must be at most 32GB'),
  port: z.number().min(1, 'Port must be at least 1').max(65535, 'Port must be at most 65535'),
  
  // Mods & Modpacks
  modpack: z.object({
    id: z.string().optional(),
    provider: z.enum(['modrinth', 'curseforge']).optional(),
    version: z.string().optional(),
  }).optional(),
  individualMods: z.array(z.object({
    id: z.string(),
    provider: z.enum(['modrinth', 'curseforge']),
    version: z.string(),
  })).optional(),
  
  // World & Performance
  levelSeed: z.string().optional(),
  levelType: z.enum(['default', 'flat', 'large_biomes', 'amplified', 'single_biome_surface']).optional(),
  generateStructures: z.boolean().optional(),
  allowNether: z.boolean().optional(),
  allowEnd: z.boolean().optional(),
  difficulty: z.enum(['peaceful', 'easy', 'normal', 'hard']).optional(),
  hardcore: z.boolean().optional(),
  pvp: z.boolean().optional(),
  onlineMode: z.boolean().optional(),
  maxPlayers: z.number().min(1, 'Max players must be at least 1').max(1000, 'Max players must be at most 1000').optional(),
  viewDistance: z.number().min(3, 'View distance must be at least 3').max(32, 'View distance must be at most 32').optional(),
  simulationDistance: z.number().min(3, 'Simulation distance must be at least 3').max(32, 'Simulation distance must be at most 32').optional(),
  enableCommandBlock: z.boolean().optional(),
  enableQuery: z.boolean().optional(),
  enableRcon: z.boolean().optional(),
  rconPort: z.number().min(1, 'RCON port must be at least 1').max(65535, 'RCON port must be at most 65535').optional(),
  rconPassword: z.string().optional(),
  motd: z.string().max(59, 'MOTD must be at most 59 characters').optional(),
  
  // Advanced
  jvmArgs: z.string().optional(),
  autoStart: z.boolean().optional(),
  autoRestart: z.boolean().optional(),
  backupEnabled: z.boolean().optional(),
  backupInterval: z.number().min(1, 'Backup interval must be at least 1 hour').max(168, 'Backup interval must be at most 168 hours').optional(),
  backupRetention: z.number().min(1, 'Backup retention must be at least 1').max(365, 'Backup retention must be at most 365').optional(),
});

describe('Server Form Data Schema', () => {
  it('should validate a complete valid server configuration', () => {
    const validData = {
      name: 'test-server',
      minecraftVersion: '1.20.1',
      loader: 'vanilla' as const,
      memory: 2048,
      port: 25565,
      levelSeed: 'test-seed',
      levelType: 'default' as const,
      generateStructures: true,
      allowNether: true,
      allowEnd: true,
      difficulty: 'normal' as const,
      hardcore: false,
      pvp: true,
      onlineMode: true,
      maxPlayers: 20,
      viewDistance: 10,
      simulationDistance: 10,
      enableCommandBlock: false,
      enableQuery: false,
      enableRcon: false,
      motd: 'A Minecraft Server',
      jvmArgs: '-Xmx2G -Xms1G',
      autoStart: false,
      autoRestart: false,
      backupEnabled: true,
      backupInterval: 24,
      backupRetention: 7,
    };

    const result = ServerFormDataSchema.safeParse(validData);
    expect(result.success).toBe(true);
  });

  it('should validate a minimal valid server configuration', () => {
    const minimalData = {
      name: 'minimal-server',
      minecraftVersion: '1.20.1',
      loader: 'vanilla' as const,
      memory: 1024,
      port: 25565,
    };

    const result = ServerFormDataSchema.safeParse(minimalData);
    expect(result.success).toBe(true);
  });

  it('should reject invalid server names', () => {
    const invalidNames = ['', 'a'.repeat(51), 'server with spaces', 'server/with/slashes'];
    
    invalidNames.forEach(name => {
      const data = {
        name,
        minecraftVersion: '1.20.1',
        loader: 'vanilla' as const,
        memory: 1024,
        port: 25565,
      };
      
      const result = ServerFormDataSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });

  it('should reject invalid memory values', () => {
    const invalidMemory = [0, 511, 32769];
    
    invalidMemory.forEach(memory => {
      const data = {
        name: 'test-server',
        minecraftVersion: '1.20.1',
        loader: 'vanilla' as const,
        memory,
        port: 25565,
      };
      
      const result = ServerFormDataSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });

  it('should reject invalid port values', () => {
    const invalidPorts = [0, 65536];
    
    invalidPorts.forEach(port => {
      const data = {
        name: 'test-server',
        minecraftVersion: '1.20.1',
        loader: 'vanilla' as const,
        memory: 1024,
        port,
      };
      
      const result = ServerFormDataSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });

  it('should reject invalid loader values', () => {
    const data = {
      name: 'test-server',
      minecraftVersion: '1.20.1',
      loader: 'invalid' as any,
      memory: 1024,
      port: 25565,
    };
    
    const result = ServerFormDataSchema.safeParse(data);
    expect(result.success).toBe(false);
  });

  it('should validate modpack configuration', () => {
    const dataWithModpack = {
      name: 'modded-server',
      minecraftVersion: '1.20.1',
      loader: 'fabric' as const,
      memory: 2048,
      port: 25565,
      modpack: {
        id: 'test-modpack',
        provider: 'modrinth' as const,
        version: '1.0.0',
      },
    };
    
    const result = ServerFormDataSchema.safeParse(dataWithModpack);
    expect(result.success).toBe(true);
  });

  it('should validate individual mods configuration', () => {
    const dataWithMods = {
      name: 'modded-server',
      minecraftVersion: '1.20.1',
      loader: 'fabric' as const,
      memory: 2048,
      port: 25565,
      individualMods: [
        {
          id: 'jei',
          provider: 'modrinth' as const,
          version: '1.0.0',
        },
        {
          id: 'fabric-api',
          provider: 'modrinth' as const,
          version: '1.0.0',
        },
      ],
    };
    
    const result = ServerFormDataSchema.safeParse(dataWithMods);
    expect(result.success).toBe(true);
  });

  it('should reject invalid max players values', () => {
    const invalidMaxPlayers = [0, 1001];
    
    invalidMaxPlayers.forEach(maxPlayers => {
      const data = {
        name: 'test-server',
        minecraftVersion: '1.20.1',
        loader: 'vanilla' as const,
        memory: 1024,
        port: 25565,
        maxPlayers,
      };
      
      const result = ServerFormDataSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });

  it('should reject invalid view distance values', () => {
    const invalidViewDistances = [2, 33];
    
    invalidViewDistances.forEach(viewDistance => {
      const data = {
        name: 'test-server',
        minecraftVersion: '1.20.1',
        loader: 'vanilla' as const,
        memory: 1024,
        port: 25565,
        viewDistance,
      };
      
      const result = ServerFormDataSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });

  it('should reject invalid backup interval values', () => {
    const invalidBackupIntervals = [0, 169];
    
    invalidBackupIntervals.forEach(backupInterval => {
      const data = {
        name: 'test-server',
        minecraftVersion: '1.20.1',
        loader: 'vanilla' as const,
        memory: 1024,
        port: 25565,
        backupInterval,
      };
      
      const result = ServerFormDataSchema.safeParse(data);
      expect(result.success).toBe(false);
    });
  });
});

// Test API response schemas
const ApiResponseSchema = z.object({
  success: z.boolean(),
  data: z.any().optional(),
  error: z.string().optional(),
  timestamp: z.string(),
});

describe('API Response Schema', () => {
  it('should validate successful API response', () => {
    const successResponse = {
      success: true,
      data: { id: 'test-id', name: 'test-server' },
      timestamp: new Date().toISOString(),
    };
    
    const result = ApiResponseSchema.safeParse(successResponse);
    expect(result.success).toBe(true);
  });

  it('should validate error API response', () => {
    const errorResponse = {
      success: false,
      error: 'Server not found',
      timestamp: new Date().toISOString(),
    };
    
    const result = ApiResponseSchema.safeParse(errorResponse);
    expect(result.success).toBe(true);
  });

  it('should reject invalid API response', () => {
    const invalidResponse = {
      success: 'yes', // Should be boolean
      data: { id: 'test-id' },
      // Missing timestamp
    };
    
    const result = ApiResponseSchema.safeParse(invalidResponse);
    expect(result.success).toBe(false);
  });
});

// Test GPU metrics schema
const GpuMetricsSchema = z.object({
  utilization: z.number().min(0).max(1),
  memory_used: z.number().min(0),
  memory_total: z.number().min(0),
  temperature: z.number().min(0),
  power_usage: z.number().min(0),
  last_update: z.string(),
});

describe('GPU Metrics Schema', () => {
  it('should validate valid GPU metrics', () => {
    const validMetrics = {
      utilization: 0.75,
      memory_used: 2048,
      memory_total: 8192,
      temperature: 65.5,
      power_usage: 150.0,
      last_update: new Date().toISOString(),
    };
    
    const result = GpuMetricsSchema.safeParse(validMetrics);
    expect(result.success).toBe(true);
  });

  it('should reject invalid GPU metrics', () => {
    const invalidMetrics = {
      utilization: 1.5, // Should be <= 1
      memory_used: -100, // Should be >= 0
      memory_total: 0, // Should be > 0
      temperature: -10, // Should be >= 0
      power_usage: -50, // Should be >= 0
      last_update: 'invalid-date', // Should be valid ISO string
    };
    
    const result = GpuMetricsSchema.safeParse(invalidMetrics);
    expect(result.success).toBe(false);
  });
});
