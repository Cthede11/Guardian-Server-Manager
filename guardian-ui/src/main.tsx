import React from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider } from 'react-router-dom';
import { router } from './app/routes';
import { liveStore } from './store/live';
import { useServersStore } from './store/servers';
import { fileManager } from './lib/file-manager';
import { settingsManager } from './lib/settings-manager';
import { errorHandler } from './lib/error-handler';
import { metricsCollector } from './lib/metrics-collector';
import { backupManager } from './lib/backup-manager';
import config from './lib/config';
import './index.css';

// Initialize all systems
async function initializeApp() {
  try {
    // Initialize error handling first
    await errorHandler.initialize();
    
    // Initialize file manager
    await fileManager.initialize();
    
    // Initialize settings manager
    await settingsManager.initialize();
    
    // Initialize backup manager
    await backupManager.loadSchedules();
    
    // Initialize stores with empty state for real backend connection
    initializeStores();
    
    // Test backend connection
    await testBackendConnection();
    
    console.log('Guardian app initialized successfully');
  } catch (error) {
    console.error('Failed to initialize Guardian app:', error);
    errorHandler.handleError(error as Error, 'App Initialization', {}, 'critical');
  }
}

// Test backend connection
async function testBackendConnection() {
  try {
    // Import waitForBackend and getAPI_BASE from the updated api module
    const { waitForBackend, getAPI_BASE } = await import('./lib/api');
    const base = await waitForBackend(15000);
    console.log('✅ Backend ready at:', base);
  } catch (error) {
    console.error('❌ Backend not reachable:', error);
  }
}

// Initialize stores with empty state for real backend connection
function initializeStores() {
  // Initialize live store with empty state
  liveStore.setState({
    console: {},
    players: {},
    freezes: {},
    pregenJobs: {},
    metrics: {},
    connected: false,
    connectionType: 'disconnected'
  });
  
  // Initialize servers store with empty state
  useServersStore.setState({
    servers: [],
    selectedServerId: null,
    serverHealth: {},
    serverSettings: {},
    loading: false,
    error: null,
  });
}

// Initialize app and render
initializeApp().then(() => {
  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode>
  );
});
