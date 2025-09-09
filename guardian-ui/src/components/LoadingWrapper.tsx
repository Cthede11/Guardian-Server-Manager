import React from 'react';
import { Loader2 } from 'lucide-react';

interface LoadingWrapperProps {
  isLoading: boolean;
  error?: string | null;
  children?: React.ReactNode;
  fallback?: React.ReactNode;
  className?: string;
}

export const LoadingWrapper: React.FC<LoadingWrapperProps> = ({
  isLoading,
  error,
  children,
  fallback,
  className = ''
}) => {
  if (error) {
    return (
      <div className={`p-6 ${className}`}>
        <div className="text-center py-12">
          <div className="text-red-500 mb-4">
            <Loader2 className="h-8 w-8 animate-spin mx-auto" />
          </div>
          <h3 className="text-lg font-semibold mb-2">Connection Error</h3>
          <p className="text-muted-foreground mb-4">{error}</p>
          <p className="text-sm text-muted-foreground">
            Please ensure the backend server is running on 127.0.0.1:52100-52150
          </p>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className={`p-6 ${className}`}>
        <div className="text-center py-12">
          <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4" />
          <p className="text-muted-foreground">Loading data...</p>
        </div>
      </div>
    );
  }

  return <>{children}</>;
};

export default LoadingWrapper;
