import { http, HttpResponse } from 'msw';
import { ServerSummary, ServerHealth } from '@/lib/types';

// Mock server data
const mockServers: ServerSummary[] = [
  {
    id: '1',
    name: 'Creative World',
    status: 'running',
    tps: 20.0,
    tickP95: 45.2,
    heapMb: 2048,
    playersOnline: 12,
    gpuQueueMs: 5.2,
    lastSnapshotAt: '2024-01-15T10:30:00Z',
    blueGreen: {
      active: 'blue',
      candidateHealthy: true,
    },
  },
  {
    id: '2',
    name: 'Survival Server',
    status: 'stopped',
    tps: 0,
    tickP95: 0,
    heapMb: 0,
    playersOnline: 0,
    gpuQueueMs: 0,
    blueGreen: {
      active: 'green',
      candidateHealthy: false,
    },
  },
  {
    id: '3',
    name: 'Modded Test',
    status: 'starting',
    tps: 0,
    tickP95: 0,
    heapMb: 1024,
    playersOnline: 0,
    gpuQueueMs: 0,
    blueGreen: {
      active: 'blue',
      candidateHealthy: true,
    },
  },
];

const mockServerHealth: Record<string, ServerHealth> = {
  '1': {
    rcon: true,
    query: true,
    crashTickets: 0,
    freezeTickets: 2,
  },
  '2': {
    rcon: false,
    query: false,
    crashTickets: 0,
    freezeTickets: 0,
  },
  '3': {
    rcon: false,
    query: false,
    crashTickets: 0,
    freezeTickets: 0,
  },
};

export const handlers = [
  // Get all servers
  http.get('/api/v1/servers', () => {
    return HttpResponse.json(mockServers);
  }),

  // Create server
  http.post('/api/v1/servers', async ({ request }) => {
    const body = await request.json() as any;
    const newServer: ServerSummary = {
      id: String(mockServers.length + 1),
      name: body.name,
      status: 'stopped',
      tps: 0,
      tickP95: 0,
      heapMb: 0,
      playersOnline: 0,
      gpuQueueMs: 0,
      blueGreen: {
        active: 'blue',
        candidateHealthy: false,
      },
    };
    mockServers.push(newServer);
    return HttpResponse.json(newServer);
  }),

  // Get server summary
  http.get('/api/v1/servers/:id/summary', ({ params }) => {
    const server = mockServers.find(s => s.id === params.id);
    if (!server) {
      return new HttpResponse(null, { status: 404 });
    }
    return HttpResponse.json(server);
  }),

  // Get server health
  http.get('/api/v1/servers/:id/health', ({ params }) => {
    const health = mockServerHealth[params.id as string];
    if (!health) {
      return new HttpResponse(null, { status: 404 });
    }
    return HttpResponse.json(health);
  }),

  // Server actions
  http.post('/api/v1/servers/:id/actions/:action', ({ params }) => {
    const { id, action } = params;
    const server = mockServers.find(s => s.id === id);
    
    if (!server) {
      return new HttpResponse(null, { status: 404 });
    }

    // Simulate action
    switch (action) {
      case 'start':
        server.status = 'starting';
        // Simulate starting process
        setTimeout(() => {
          server.status = 'running';
          server.tps = 20.0;
        }, 2000);
        break;
      case 'stop':
        server.status = 'stopping';
        // Simulate stopping process
        setTimeout(() => {
          server.status = 'stopped';
          server.tps = 0;
          server.playersOnline = 0;
        }, 1000);
        break;
      case 'restart':
        server.status = 'stopping';
        // Simulate restart process
        setTimeout(() => {
          server.status = 'starting';
          setTimeout(() => {
            server.status = 'running';
            server.tps = 20.0;
          }, 2000);
        }, 1000);
        break;
      case 'promote':
        server.blueGreen.active = server.blueGreen.active === 'blue' ? 'green' : 'blue';
        break;
    }

    return HttpResponse.json({ ok: true });
  }),

  // Console command
  http.post('/api/v1/servers/:id/console/command', async ({ request }) => {
    const body = await request.json() as any;
    console.log(`Console command: ${body.cmd}`);
    return HttpResponse.json({ ok: true });
  }),

  // Get online players
  http.get('/api/v1/servers/:id/players/online', () => {
    return HttpResponse.json([
      {
        uuid: '1',
        name: 'Player1',
        online: true,
        lastSeen: new Date().toISOString(),
        playtime: 3600,
      },
      {
        uuid: '2',
        name: 'Player2',
        online: true,
        lastSeen: new Date().toISOString(),
        playtime: 7200,
      },
    ]);
  }),

  // Get world heatmap
  http.get('/api/v1/servers/:id/world/heatmap', () => {
    return HttpResponse.json({
      cells: Array.from({ length: 100 }, (_, i) => ({
        x: Math.floor(i / 10) - 5,
        z: (i % 10) - 5,
        value: Math.random() * 100,
      })),
    });
  }),

  // Get freezes
  http.get('/api/v1/servers/:id/freezes', () => {
    return HttpResponse.json([
      {
        id: '1',
        actorId: 'entity-123',
        location: {
          x: 100,
          y: 64,
          z: 200,
          dimension: 'minecraft:overworld',
        },
        duration: 5000,
        createdAt: new Date().toISOString(),
      },
    ]);
  }),

  // Thaw freeze
  http.post('/api/v1/servers/:id/thaw/:actorId', () => {
    return HttpResponse.json({ ok: true });
  }),

  // Get mods
  http.get('/api/v1/servers/:id/mods', () => {
    return HttpResponse.json([
      {
        id: 'mod1',
        name: 'JEI',
        version: '1.20.1-12.0.0',
        enabled: true,
      },
      {
        id: 'mod2',
        name: 'OptiFine',
        version: '1.20.1_HD_U_I1',
        enabled: true,
      },
    ]);
  }),

  // Get conflicts
  http.get('/api/v1/servers/:id/compat/conflicts', () => {
    return HttpResponse.json([
      {
        id: 'conflict1',
        mods: ['mod1', 'mod2'],
        severity: 'warning',
        description: 'Potential performance impact',
      },
    ]);
  }),

  // Get rules
  http.get('/api/v1/servers/:id/rules', () => {
    return HttpResponse.json([
      {
        id: 'rule1',
        name: 'Anti-Lag Rule',
        enabled: true,
        description: 'Prevents lag-causing entities',
        code: 'if (entity.tickTime > 50) { entity.remove(); }',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
    ]);
  }),

  // Get metrics
  http.get('/api/v1/servers/:id/metrics', () => {
    const now = Date.now();
    const tps = Array.from({ length: 60 }, (_, i) => ({
      timestamp: new Date(now - (60 - i) * 1000).toISOString(),
      value: 20 + Math.random() * 2 - 1,
    }));

    return HttpResponse.json({
      tps,
      tickPhases: {
        entity: Array.from({ length: 60 }, (_, i) => ({
          timestamp: new Date(now - (60 - i) * 1000).toISOString(),
          value: 10 + Math.random() * 5,
        })),
        tile: Array.from({ length: 60 }, (_, i) => ({
          timestamp: new Date(now - (60 - i) * 1000).toISOString(),
          value: 5 + Math.random() * 3,
        })),
        world: Array.from({ length: 60 }, (_, i) => ({
          timestamp: new Date(now - (60 - i) * 1000).toISOString(),
          value: 3 + Math.random() * 2,
        })),
      },
      heap: Array.from({ length: 60 }, (_, i) => ({
        timestamp: new Date(now - (60 - i) * 1000).toISOString(),
        value: 2048 + Math.random() * 512,
      })),
      gpuLatency: Array.from({ length: 60 }, (_, i) => ({
        timestamp: new Date(now - (60 - i) * 1000).toISOString(),
        value: 5 + Math.random() * 3,
      })),
    });
  }),

  // Get snapshots
  http.get('/api/v1/servers/:id/snapshots', () => {
    return HttpResponse.json([
      {
        id: 'snapshot1',
        name: 'Daily Backup',
        size: 1024 * 1024 * 1024, // 1GB
        createdAt: '2024-01-15T10:30:00Z',
        scope: 'global',
        status: 'ready',
      },
    ]);
  }),

  // Create snapshot
  http.post('/api/v1/servers/:id/snapshots/create', () => {
    return HttpResponse.json({
      id: 'snapshot2',
      name: 'Manual Backup',
      size: 0,
      createdAt: new Date().toISOString(),
      scope: 'global',
      status: 'creating',
    });
  }),

  // Get events
  http.get('/api/v1/servers/:id/events', () => {
    return HttpResponse.json([
      {
        id: 'event1',
        name: 'Daily Restart',
        description: 'Automatic server restart',
        scheduledAt: '2024-01-16T03:00:00Z',
        status: 'scheduled',
        actions: ['backup', 'restart'],
      },
    ]);
  }),

  // Get pregen status
  http.get('/api/v1/servers/:id/pregen/status', () => {
    return HttpResponse.json({
      jobs: [
        {
          id: 'pregen1',
          region: { x: 0, z: 0, radius: 1000 },
          dimension: 'minecraft:overworld',
          priority: 'normal',
          status: 'running',
          progress: 45,
          eta: '2h 30m',
          gpuAssist: true,
        },
      ],
    });
  }),

  // Get sharding topology
  http.get('/api/v1/sharding/topology', () => {
    return HttpResponse.json({
      shards: [
        {
          id: 'shard1',
          name: 'Overworld Shard',
          status: 'healthy',
          dimensions: ['minecraft:overworld'],
        },
        {
          id: 'shard2',
          name: 'Nether Shard',
          status: 'healthy',
          dimensions: ['minecraft:the_nether'],
        },
      ],
    });
  }),

  // Get diagnostics
  http.get('/api/v1/servers/:id/diagnostics', () => {
    return HttpResponse.json({
      crashes: [],
      dumps: [],
      gcLogs: [],
    });
  }),

  // Create diagnostic bundle
  http.post('/api/v1/servers/:id/diagnostics/bundle', () => {
    return HttpResponse.json({
      url: 'https://example.com/diagnostics/bundle.zip',
    });
  }),

  // Get server settings
  http.get('/api/v1/servers/:id/settings', () => {
    return HttpResponse.json({
      general: {
        name: 'My Server',
        description: 'A modded Minecraft server',
        version: '1.20.1',
        loader: 'forge',
      },
      jvm: {
        memory: 4096,
        flags: ['-XX:+UseG1GC'],
      },
      gpu: {
        enabled: true,
        queueSize: 1000,
      },
      ha: {
        enabled: false,
        blueGreen: true,
      },
      paths: {
        world: '/opt/minecraft/world',
        mods: '/opt/minecraft/mods',
        config: '/opt/minecraft/config',
      },
      composer: {
        profile: 'balanced',
      },
      tokens: {
        rcon: '••••••••••••••••',
        query: '••••••••••••••••',
      },
    });
  }),

  // Update server settings
  http.put('/api/v1/servers/:id/settings', async ({ request }) => {
    const body = await request.json();
    return HttpResponse.json(body);
  }),

  // Get workspace settings
  http.get('/api/v1/workspace/settings', () => {
    return HttpResponse.json({
      users: [
        {
          id: '1',
          name: 'admin',
          role: 'admin',
        },
      ],
      backupTargets: [
        {
          id: '1',
          name: 'Local Storage',
          type: 'local',
          config: { path: '/opt/backups' },
        },
      ],
      apiTokens: [
        {
          id: '1',
          name: 'Web UI Token',
          token: '••••••••••••••••',
          createdAt: '2024-01-01T00:00:00Z',
        },
      ],
      theme: {
        mode: 'dark',
        accent: 'blue',
      },
    });
  }),

  // Update workspace settings
  http.put('/api/v1/workspace/settings', async ({ request }) => {
    const body = await request.json();
    return HttpResponse.json(body);
  }),
];
