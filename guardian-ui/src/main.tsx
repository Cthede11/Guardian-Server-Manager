import React from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider } from 'react-router-dom';
import { router } from './app/routes';
import { useLive } from './store/live-new';
import { useServers } from './store/servers-new';
import { fileManager } from './lib/file-manager';
import { settingsManager } from './lib/settings-manager';
import { errorHandler } from './lib/error-handler';
import { metricsCollector } from './lib/metrics-collector';
import { backupManager } from './lib/backup-manager';
import config from './lib/config';
import './index.css';

// Global initialization flag to prevent multiple backend starts
let isInitializing = false;
let isInitialized = false;

// Initialize all systems
async function initializeApp() {
  try {
    // Prevent multiple initialization attempts
    if (isInitializing || isInitialized) {
      console.log('App initialization already in progress or completed');
      return;
    }
    
    isInitializing = true;
    
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
    
    // Test backend connection with delay to ensure Tauri is ready
    setTimeout(async () => {
      await testBackendConnection();
    }, 1000);
    
    isInitialized = true;
    isInitializing = false;
    console.log('Guardian app initialized successfully');
  } catch (error) {
    isInitializing = false;
    console.error('Failed to initialize Guardian app:', error);
    errorHandler.handleError(error as Error, 'App Initialization', {}, 'critical');
  }
}

// Global backend connection flag
let isConnectingToBackend = false;

// Test backend connection
async function testBackendConnection() {
  try {
    // Prevent multiple backend connection attempts
    if (isConnectingToBackend) {
      console.log('Backend connection already in progress, skipping...');
      return;
    }
    
    isConnectingToBackend = true;
    console.log('🔍 Testing backend connection...');
    console.log('🔍 Tauri context available:', typeof window !== 'undefined' && (window as any).__TAURI__);
    
    // Check if we're in Tauri context
    if (typeof window !== 'undefined' && (window as any).__TAURI__) {
      console.log('🔍 Attempting to call start_backend command...');
      // Use Tauri start_backend command
      const { invoke } = await import('@tauri-apps/api/core');
      const baseFromTauri = await invoke('start_backend') as string;
      console.log('✅ Backend started via Tauri sidecar:', baseFromTauri);
      
      // Update the API base URL with the correct backend URL
      if (baseFromTauri) {
        // Update the API client configuration
        const { updateApiBase } = await import('./lib/api');
        await updateApiBase(baseFromTauri);
        console.log('✅ API base URL updated to:', baseFromTauri);
      }
      
      // Initialize servers list using Tauri commands instead of HTTP
      const { useServers } = await import('./store/servers-new');
      try {
        await useServers.getState().fetchServers();
        console.log('✅ Servers list initialized successfully');
      } catch (error) {
        console.log('⚠️ Servers list initialization failed, but continuing:', error);
      }
    } else {
      console.log('⚠️ Not in Tauri context, skipping backend connection test');
    }
    
    isConnectingToBackend = false;
    return;
  } catch (error) {
    isConnectingToBackend = false;
    console.error('❌ Backend not reachable:', error);
    console.error('Error details:', error);
  }
}

// Initialize stores with empty state for real backend connection
function initializeStores() {
  // Initialize live store with empty state
  useLive.setState({
    console: {},
    players: {},
    freezes: {},
    pregenJobs: {},
    metrics: {},
    health: {},
  });
  
  // Initialize servers store with empty state
  useServers.setState({
    selectedId: undefined,
    summaries: {},
    settings: {},
    health: {},
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
