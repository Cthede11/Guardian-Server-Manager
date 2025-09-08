import { RouterProvider } from 'react-router-dom';
import { router } from './app/routes';
import { Toaster } from './components/ui/toaster';
import { ErrorBoundary } from './components/ui/ErrorBoundary';
import { RealtimeProvider } from './lib/realtime-provider';
import './App.css';

function App() {
  return (
    <ErrorBoundary>
      <RealtimeProvider>
        <div className="min-h-screen bg-background">
          <RouterProvider router={router} />
          <Toaster />
        </div>
      </RealtimeProvider>
    </ErrorBoundary>
  );
}

export default App;
