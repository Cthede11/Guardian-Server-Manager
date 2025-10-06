import React from 'react';
import { Outlet, useParams } from 'react-router-dom';
import { Sidebar } from './Sidebar';
import { ServerHeader, ServerTabs } from './ServerHeader';
import { ErrorBoundary } from '@/components/ui/ErrorBoundary';
import { Toaster } from '@/components/ui/toaster';
// import { realtimeProvider } from '@/lib/realtime-provider';
import { ConnectionStatus } from '@/components/ConnectionStatus';

export const AppShell: React.FC = () => {
  // const { id: serverId } = useParams<{ id: string }>();

  // Start/stop real-time monitoring based on server selection
  // Real-time monitoring is handled by individual components

  return (
    <ErrorBoundary>
      <div className="flex h-screen bg-background text-foreground">
        {/* Left Sidebar */}
        <Sidebar />
        
        {/* Main Content Area */}
        <div className="flex-1 flex flex-col">
          {/* Header */}
          <ServerHeader />
          
          {/* Server Tabs */}
          <ServerTabs />
          
          {/* Connection Status */}
          <ConnectionStatus />
          
          {/* Content */}
          <main className="flex-1 overflow-auto p-6">
            <div className="animate-fade-in-up">
              <Outlet />
            </div>
          </main>
        </div>
      </div>
      
      {/* Toast Notifications */}
      <Toaster />
    </ErrorBoundary>
  );
};

export default AppShell;
