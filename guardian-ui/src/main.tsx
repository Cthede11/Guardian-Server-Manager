import React from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider } from 'react-router-dom';
import { router } from './app/routes';
import { liveStore } from './store/live';
import { useServersStore } from './store/servers';
import { realDataService } from './lib/real-data';
import './index.css';

// Start MSW in development (only if backend is not available)
async function startMSW() {
  if (import.meta.env.DEV && import.meta.env.VITE_USE_MOCK_DATA === 'true') {
    const { worker } = await import('./mocks/browser');
    await worker.start({
      onUnhandledRequest: 'bypass',
      serviceWorker: {
        url: '/mockServiceWorker.js',
      },
    });
  }
}

// Initialize live store with mock data for development (only if using mock data)
function initializeLiveStore() {
  if (import.meta.env.DEV && import.meta.env.VITE_USE_MOCK_DATA === 'true') {
    const mockConsoleMessages = [
      {
        ts: new Date().toISOString(),
        level: 'info' as const,
        msg: 'Server started successfully'
      },
      {
        ts: new Date().toISOString(),
        level: 'info' as const,
        msg: 'Player Player1 joined the game'
      }
    ];

    const mockPlayers = [
      {
        uuid: '1',
        name: 'Player1',
        online: true,
        lastSeen: new Date().toISOString(),
        playtime: 3600
      },
      {
        uuid: '2',
        name: 'Player2',
        online: true,
        lastSeen: new Date().toISOString(),
        playtime: 7200
      }
    ];

    const mockFreezes = [
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
      }
    ];

    const mockPregenJobs = [
      {
        id: 'pregen1',
        region: { x: 0, z: 0, radius: 1000 },
        dimension: 'minecraft:overworld',
        priority: 'normal' as const,
        status: 'running' as const,
        progress: 45,
        eta: '2h 30m',
        gpuAssist: true
      }
    ];

    // Create separate metric objects for each server to avoid shared references
    const createMockMetrics = () => ({
      tps: [
        { timestamp: Date.now(), value: 20.0 }
      ],
      heap: [{ timestamp: Date.now(), value: 2048 }],
      tickP95: [{ timestamp: Date.now(), value: 45.2 }],
      gpuMs: [{ timestamp: Date.now(), value: 5.2 }]
    });

    // Populate the live store with mock data
    liveStore.setState({
      console: {
        '1': [...mockConsoleMessages],
        '2': [...mockConsoleMessages],
        '3': [...mockConsoleMessages]
      },
      players: {
        '1': [...mockPlayers],
        '2': [...mockPlayers],
        '3': [...mockPlayers]
      },
      freezes: {
        '1': [...mockFreezes],
        '2': [...mockFreezes],
        '3': [...mockFreezes]
      },
      pregenJobs: {
        '1': [...mockPregenJobs],
        '2': [...mockPregenJobs],
        '3': [...mockPregenJobs]
      },
      metrics: {
        '1': createMockMetrics(),
        '2': createMockMetrics(),
        '3': createMockMetrics()
      },
      connected: true,
      connectionType: 'socket'
    });
  }
}

// Initialize servers store with mock data for development (only if using mock data)
function initializeServersStore() {
  if (import.meta.env.DEV && import.meta.env.VITE_USE_MOCK_DATA === 'true') {
    const mockServers = [
      {
        id: '1',
        name: 'Creative World',
        status: 'running' as const,
        tps: 20.0,
        tickP95: 45.2,
        heapMb: 2048,
        playersOnline: 12,
        gpuQueueMs: 5.2,
        lastSnapshotAt: '2024-01-15T10:30:00Z',
        blueGreen: {
          active: 'blue' as const,
          candidateHealthy: true,
        },
      },
      {
        id: '2',
        name: 'Survival Server',
        status: 'stopped' as const,
        tps: 0,
        tickP95: 0,
        heapMb: 0,
        playersOnline: 0,
        gpuQueueMs: 0,
        blueGreen: {
          active: 'green' as const,
          candidateHealthy: false,
        },
      },
      {
        id: '3',
        name: 'Modded Test',
        status: 'starting' as const,
        tps: 0,
        tickP95: 0,
        heapMb: 1024,
        playersOnline: 0,
        gpuQueueMs: 0,
        blueGreen: {
          active: 'blue' as const,
          candidateHealthy: true,
        },
      },
    ];

    // Populate the servers store with mock data
    useServersStore.setState({
      servers: mockServers,
      selectedServerId: null,
      serverHealth: {},
      serverSettings: {},
      loading: false,
      error: null,
    });
  }
}

// Initialize MSW and then render the app
startMSW().then(async () => {
  // Initialize stores with mock data (only if using mock data)
  if (import.meta.env.VITE_USE_MOCK_DATA === 'true') {
    initializeLiveStore();
    initializeServersStore();
  } else {
    // Use real data service
    try {
      await realDataService.start();
      console.log('Real data service started successfully');
    } catch (error) {
      console.error('Failed to start real data service:', error);
      // Fallback to mock data if real data fails
      initializeLiveStore();
      initializeServersStore();
    }
  }
  
  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>
  );
});