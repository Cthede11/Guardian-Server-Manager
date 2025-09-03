import React from 'react';
import { Outlet } from 'react-router-dom';
import { Sidebar } from './Sidebar';
import { ServerHeader, ServerTabs } from './ServerHeader';
import { ErrorBoundary } from '@/components/ui/ErrorBoundary';
import { Toaster } from '@/components/ui/toaster';

export const AppShell: React.FC = () => {
  return (
    <ErrorBoundary>
      <div className="flex h-screen bg-background">
        {/* Left Sidebar */}
        <Sidebar />
        
        {/* Main Content Area */}
        <div className="flex-1 flex flex-col">
          {/* Header */}
          <ServerHeader />
          
          {/* Server Tabs */}
          <ServerTabs />
          
          {/* Content */}
          <main className="flex-1 overflow-auto">
            <Outlet />
          </main>
        </div>
      </div>
      
      {/* Toast Notifications */}
      <Toaster />
    </ErrorBoundary>
  );
};

export default AppShell;
