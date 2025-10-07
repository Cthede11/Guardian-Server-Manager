import React, { useState, useEffect } from 'react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Wifi, WifiOff, RefreshCw, AlertTriangle } from 'lucide-react';
import { apiClient as api } from '@/lib/api';

interface ConnectionStatusProps {
  className?: string;
}

export const ConnectionStatus: React.FC<ConnectionStatusProps> = ({ className = '' }) => {
  const [isConnected, setIsConnected] = useState<boolean | null>(null);
  const [isChecking, setIsChecking] = useState(false);
  const [lastError, setLastError] = useState<string | null>(null);

  const checkConnection = async () => {
    setIsChecking(true);
    try {
      const response = await api.getServersWithStatus();
      // Check for success property from API response
      const isSuccess = response.success === true;
      setIsConnected(isSuccess);
      setLastError(isSuccess ? null : response.error || 'Unknown error');
    } catch (error) {
      setIsConnected(false);
      setLastError(error instanceof Error ? error.message : 'Network error');
    } finally {
      setIsChecking(false);
    }
  };

  useEffect(() => {
    checkConnection();
    
    // Check connection every 30 seconds
    const interval = setInterval(checkConnection, 30000);
    return () => clearInterval(interval);
  }, []);

  if (isConnected === null) {
    return null; // Don't show anything while checking
  }

  if (isConnected) {
    return null; // Don't show anything when connected
  }

  return (
    <div className={`p-4 ${className}`}>
      <Alert variant="destructive">
        <WifiOff className="h-4 w-4" />
        <AlertDescription className="flex items-center justify-between">
          <div>
            <strong>Backend Connection Failed</strong>
            <p className="text-sm mt-1">
              {lastError || 'Cannot connect to the backend server. Please ensure the backend is running on 127.0.0.1:52100-52150.'}
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={checkConnection}
            disabled={isChecking}
            className="ml-4"
          >
            {isChecking ? (
              <RefreshCw className="h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
            {isChecking ? 'Checking...' : 'Retry'}
          </Button>
        </AlertDescription>
      </Alert>
    </div>
  );
};

export default ConnectionStatus;
