import { z } from 'zod';

// Server types
export const ServerStatusSchema = z.enum(['stopped', 'starting', 'running', 'stopping']);
export type ServerStatus = z.infer<typeof ServerStatusSchema>;

export const ServerSummarySchema = z.object({
  id: z.string(),
  name: z.string(),
  status: ServerStatusSchema,
  tps: z.number(),
  tickP95: z.number(),
  heapMb: z.number(),
  playersOnline: z.number(),
  gpuQueueMs: z.number(),
  lastSnapshotAt: z.string().optional(),
  type: z.string().optional(),
  version: z.string().optional(),
  maxPlayers: z.number().optional(),
  memory: z.number().optional(),
  blueGreen: z.object({
    active: z.enum(['blue', 'green']),
    candidateHealthy: z.boolean(),
  }),
});
export type ServerSummary = z.infer<typeof ServerSummarySchema>;

export const ServerHealthSchema = z.object({
  rcon: z.boolean(),
  query: z.boolean(),
  crashTickets: z.number(),
  freezeTickets: z.number(),
});
export type ServerHealth = z.infer<typeof ServerHealthSchema>;

// Console types
export const ConsoleMessageSchema = z.object({
  ts: z.string(),
  level: z.enum(['info', 'warn', 'error', 'debug']),
  msg: z.string(),
});
export type ConsoleMessage = z.infer<typeof ConsoleMessageSchema>;

// Player types
export const PlayerSchema = z.object({
  uuid: z.string(),
  name: z.string(),
  online: z.boolean(),
  lastSeen: z.string().optional(),
  playtime: z.number().optional(),
});
export type Player = z.infer<typeof PlayerSchema>;

// World types
export const HeatmapCellSchema = z.object({
  x: z.number(),
  z: z.number(),
  value: z.number(),
});
export type HeatmapCell = z.infer<typeof HeatmapCellSchema>;

export const FreezeTicketSchema = z.object({
  id: z.string(),
  actorId: z.string(),
  location: z.object({
    x: z.number(),
    y: z.number(),
    z: z.number(),
    dimension: z.string(),
  }),
  duration: z.number(),
  createdAt: z.string(),
});
export type FreezeTicket = z.infer<typeof FreezeTicketSchema>;

// Mod types
export const ModInfoSchema = z.object({
  id: z.string(),
  name: z.string(),
  version: z.string(),
  enabled: z.boolean(),
  conflicts: z.array(z.string()).optional(),
});
export type ModInfo = z.infer<typeof ModInfoSchema>;

export const ConflictSchema = z.object({
  id: z.string(),
  mods: z.array(z.string()),
  severity: z.enum(['warning', 'error']),
  description: z.string(),
});
export type Conflict = z.infer<typeof ConflictSchema>;

// Rule types
export const RuleSchema = z.object({
  id: z.string(),
  name: z.string(),
  enabled: z.boolean(),
  description: z.string(),
  code: z.string(),
  createdAt: z.string(),
  updatedAt: z.string(),
});
export type Rule = z.infer<typeof RuleSchema>;

// Performance types
export const MetricPointSchema = z.object({
  timestamp: z.string(),
  value: z.number(),
});
export type MetricPoint = z.infer<typeof MetricPointSchema>;

export const PerformanceMetricsSchema = z.object({
  tps: z.array(MetricPointSchema),
  tickPhases: z.object({
    entity: z.array(MetricPointSchema),
    tile: z.array(MetricPointSchema),
    world: z.array(MetricPointSchema),
  }),
  heap: z.array(MetricPointSchema),
  gpuLatency: z.array(MetricPointSchema),
});
export type PerformanceMetrics = z.infer<typeof PerformanceMetricsSchema>;

// Backup types
export const SnapshotSchema = z.object({
  id: z.string(),
  name: z.string(),
  size: z.number(),
  createdAt: z.string(),
  scope: z.enum(['global', 'dimension', 'claim', 'chunk']),
  status: z.enum(['creating', 'ready', 'failed']),
});
export type Snapshot = z.infer<typeof SnapshotSchema>;

// Event types
export const EventSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  scheduledAt: z.string(),
  status: z.enum(['scheduled', 'running', 'completed', 'failed']),
  actions: z.array(z.string()),
});
export type Event = z.infer<typeof EventSchema>;

// Pregen types
export const PregenJobSchema = z.object({
  id: z.string(),
  region: z.object({
    x: z.number(),
    z: z.number(),
    radius: z.number(),
  }),
  dimension: z.string(),
  priority: z.enum(['low', 'normal', 'high']),
  status: z.enum(['queued', 'running', 'completed', 'failed']),
  progress: z.number(),
  eta: z.string().optional(),
  gpuAssist: z.boolean(),
});
export type PregenJob = z.infer<typeof PregenJobSchema>;

// Sharding types
export const ShardSchema = z.object({
  id: z.string(),
  name: z.string(),
  status: z.enum(['healthy', 'degraded', 'offline']),
  dimensions: z.array(z.string()),
});
export type Shard = z.infer<typeof ShardSchema>;

export const ShardingTopologySchema = z.object({
  shards: z.array(ShardSchema),
});
export type ShardingTopology = z.infer<typeof ShardingTopologySchema>;

// Shard Assignment types
export const ShardAssignmentSchema = z.object({
  id: z.string(),
  shardId: z.string(),
  serverId: z.string(),
  dimensions: z.array(z.string()),
  playerCount: z.number(),
  status: z.enum(['active', 'inactive', 'error']),
});
export type ShardAssignment = z.infer<typeof ShardAssignmentSchema>;

// Crash Signature types
export const CrashSignatureSchema = z.object({
  id: z.string(),
  pattern: z.string(),
  severity: z.enum(['low', 'medium', 'high', 'critical']),
  description: z.string(),
  occurrences: z.number(),
  lastSeen: z.string(),
});
export type CrashSignature = z.infer<typeof CrashSignatureSchema>;

// Mod types (alias for ModInfo)
export type Mod = ModInfo;

// Settings types
export const ServerSettingsSchema = z.object({
  general: z.object({
    name: z.string(),
    description: z.string(),
    version: z.string(),
    loader: z.string(),
  }),
  jvm: z.object({
    memory: z.number(),
    flags: z.array(z.string()),
  }),
  gpu: z.object({
    enabled: z.boolean(),
    queueSize: z.number(),
  }),
  ha: z.object({
    enabled: z.boolean(),
    blueGreen: z.boolean(),
  }),
  paths: z.object({
    world: z.string(),
    mods: z.string(),
    config: z.string(),
  }),
  composer: z.object({
    profile: z.string(),
  }),
  tokens: z.object({
    rcon: z.string(),
    query: z.string(),
  }),
});
export type ServerSettings = z.infer<typeof ServerSettingsSchema>;

export const WorkspaceSettingsSchema = z.object({
  users: z.array(z.object({
    id: z.string(),
    name: z.string(),
    role: z.enum(['admin', 'operator', 'viewer']),
  })),
  backupTargets: z.array(z.object({
    id: z.string(),
    name: z.string(),
    type: z.enum(['s3', 'local']),
    config: z.record(z.string(), z.any()),
  })),
  apiTokens: z.array(z.object({
    id: z.string(),
    name: z.string(),
    token: z.string(),
    createdAt: z.string(),
  })),
  theme: z.object({
    mode: z.enum(['dark', 'light']),
    accent: z.string(),
  }),
});
export type WorkspaceSettings = z.infer<typeof WorkspaceSettingsSchema>;

// API Response types
export const ApiResponseSchema = z.object({
  ok: z.boolean(),
  data: z.any().optional(),
  error: z.string().optional(),
});

export type ApiResponse<T = any> = {
  ok: boolean;
  data?: T;
  error?: string;
};
