import React from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider } from 'react-router-dom';
import { router } from './app/routes';
import { liveStore } from './store/live';
import './index.css';

// Start MSW in development
async function startMSW() {
  if (import.meta.env.DEV) {
    const { worker } = await import('./mocks/browser');
    await worker.start({
      onUnhandledRequest: 'bypass',
      serviceWorker: {
        url: '/mockServiceWorker.js',
      },
    });
  }
}

// Initialize live store with mock data for development
function initializeLiveStore() {
  if (import.meta.env.DEV) {
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
        playtime: 3600,
        ping: 45,
        dimension: 'minecraft:overworld',
        x: 100,
        y: 64,
        z: 200
      },
      {
        uuid: '2',
        name: 'Player2',
        online: true,
        lastSeen: new Date().toISOString(),
        playtime: 7200,
        ping: 32,
        dimension: 'minecraft:overworld',
        x: -50,
        y: 70,
        z: 150
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
        name: 'Spawn Region Pregen',
        region: { centerX: 0, centerZ: 0, radius: 1000 },
        dimension: 'minecraft:overworld',
        priority: 'normal' as const,
        status: 'running' as const,
        progress: 45,
        eta: '2h 30m',
        gpuAssist: true,
        gpuAccelerated: true,
        totalChunks: 10000,
        completedChunks: 4500,
        chunksPerSecond: 25,
        memoryUsage: 1024,
        createdAt: Date.now() - 3600000,
        startTime: Date.now() - 3600000,
        estimatedTime: 120,
        createdBy: 'admin',
        tags: ['spawn', 'overworld']
      }
    ];

    const mockMetrics = {
      tps: [
        { timestamp: Date.now(), value: 20.0 }
      ],
      heap: [{ timestamp: Date.now(), value: 2048 }],
      tickP95: [{ timestamp: Date.now(), value: 45.2 }],
      gpuMs: [{ timestamp: Date.now(), value: 5.2 }]
    };

    // Populate the live store with mock data
    liveStore.setState({
      console: {
        '1': mockConsoleMessages,
        '2': mockConsoleMessages,
        '3': mockConsoleMessages
      },
      players: {
        '1': mockPlayers,
        '2': mockPlayers,
        '3': mockPlayers
      },
      freezes: {
        '1': mockFreezes,
        '2': mockFreezes,
        '3': mockFreezes
      },
      pregenJobs: {
        '1': mockPregenJobs,
        '2': mockPregenJobs,
        '3': mockPregenJobs
      },
      metrics: {
        '1': mockMetrics,
        '2': mockMetrics,
        '3': mockMetrics
      },
      connected: true,
      connectionType: 'socket'
    });
  }
}

// Initialize MSW and then render the app
startMSW().then(() => {
  // Initialize live store with mock data
  initializeLiveStore();
  
  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>
  );
});