import React, { useState, useEffect } from 'react';
import { apiClient, CircuitBreakerState } from '../lib/api-client';

interface ConnectionStatusProps {
  className?: string;
}

export const ConnectionStatus: React.FC<ConnectionStatusProps> = ({ className = '' }) => {
  const [status, setStatus] = useState<'connected' | 'disconnected' | 'checking'>('checking');
  const [circuitBreakerStatus, setCircuitBreakerStatus] = useState(apiClient.getCircuitBreakerStatus());
  const [lastError, setLastError] = useState<string | null>(null);
  const [retryCount, setRetryCount] = useState(0);

  useEffect(() => {
    const checkConnection = async () => {
      try {
        setStatus('checking');
        await apiClient.get('/api/healthz');
        setStatus('connected');
        setLastError(null);
        setRetryCount(0);
      } catch (error) {
        setStatus('disconnected');
        setLastError(error instanceof Error ? error.message : 'Unknown error');
        setRetryCount(prev => prev + 1);
      }
      
      setCircuitBreakerStatus(apiClient.getCircuitBreakerStatus());
    };

    // Check connection immediately
    checkConnection();

    // Set up periodic health checks
    const interval = setInterval(checkConnection, 30000); // Check every 30 seconds

    return () => clearInterval(interval);
  }, []);

  const getStatusColor = () => {
    switch (status) {
      case 'connected':
        return 'text-green-600';
      case 'disconnected':
        return 'text-red-600';
      case 'checking':
        return 'text-yellow-600';
      default:
        return 'text-gray-600';
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case 'connected':
        return '🟢';
      case 'disconnected':
        return '🔴';
      case 'checking':
        return '🟡';
      default:
        return '⚪';
    }
  };

  const getCircuitBreakerStatusText = () => {
    switch (circuitBreakerStatus.state) {
      case CircuitBreakerState.CLOSED:
        return 'Normal';
      case CircuitBreakerState.OPEN:
        return 'Open (Failing)';
      case CircuitBreakerState.HALF_OPEN:
        return 'Testing Recovery';
      default:
        return 'Unknown';
    }
  };

  const getCircuitBreakerColor = () => {
    switch (circuitBreakerStatus.state) {
      case CircuitBreakerState.CLOSED:
        return 'text-green-600';
      case CircuitBreakerState.OPEN:
        return 'text-red-600';
      case CircuitBreakerState.HALF_OPEN:
        return 'text-yellow-600';
      default:
        return 'text-gray-600';
    }
  };

  const handleResetCircuitBreaker = () => {
    apiClient.resetCircuitBreaker();
    setCircuitBreakerStatus(apiClient.getCircuitBreakerStatus());
  };

  return (
    <div className={`flex items-center space-x-2 ${className}`}>
      <span className="text-sm">
        {getStatusIcon()} {status === 'checking' ? 'Checking...' : status}
      </span>
      
      <div className="text-xs text-gray-500">
        Circuit Breaker: 
        <span className={`ml-1 ${getCircuitBreakerColor()}`}>
          {getCircuitBreakerStatusText()}
        </span>
        {circuitBreakerStatus.failureCount > 0 && (
          <span className="ml-1">
            ({circuitBreakerStatus.failureCount} failures)
          </span>
        )}
      </div>

      {lastError && (
        <div className="text-xs text-red-500 max-w-xs truncate" title={lastError}>
          Error: {lastError}
        </div>
      )}

      {retryCount > 0 && (
        <div className="text-xs text-yellow-600">
          Retries: {retryCount}
        </div>
      )}

      {circuitBreakerStatus.state === CircuitBreakerState.OPEN && (
        <button
          onClick={handleResetCircuitBreaker}
          className="text-xs px-2 py-1 bg-red-100 text-red-700 rounded hover:bg-red-200"
        >
          Reset
        </button>
      )}
    </div>
  );
};

export default ConnectionStatus;