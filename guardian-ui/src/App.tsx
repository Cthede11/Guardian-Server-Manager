import { RouterProvider } from 'react-router-dom';
import { router } from './app/routes';
import { Toaster } from './components/ui/toaster';
import { ErrorBoundary } from './components/ui/ErrorBoundary';
import { WorkspaceRealtimeProvider } from './components/RealtimeDataProvider';
import './App.css';

function App() {
  return (
    <ErrorBoundary>
      <WorkspaceRealtimeProvider>
        <div className="min-h-screen bg-background">
          <RouterProvider router={router} />
          <Toaster />
        </div>
      </WorkspaceRealtimeProvider>
    </ErrorBoundary>
  );
}

export default App;
