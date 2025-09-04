import React from 'react';
import { ServerCardSkeleton, PlayerRowSkeleton, StatCardSkeleton, ChartSkeleton } from './SkeletonLoader';
import { LoadingEmptyState } from './EmptyState';
import { cn } from '@/lib/utils';

interface LoadingStateProps {
  className?: string;
  message?: string;
  variant?: 'default' | 'overlay' | 'inline';
}

export const LoadingState: React.FC<LoadingStateProps> = ({
  className,
  message = 'Loading...',
  variant = 'default',
}) => {
  if (variant === 'overlay') {
    return (
      <div className={cn(
        'absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50',
        className
      )}>
        <div className="flex flex-col items-center space-y-4">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          <p className="text-sm text-muted-foreground">{message}</p>
        </div>
      </div>
    );
  }

  if (variant === 'inline') {
    return (
      <div className={cn('flex items-center space-x-2', className)}>
        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary"></div>
        <span className="text-sm text-muted-foreground">{message}</span>
      </div>
    );
  }

  return <LoadingEmptyState message={message} />;
};

// Specific loading states for different components
export const ServersListLoading: React.FC<{ count?: number }> = ({ count = 3 }) => (
  <div className="space-y-4">
    {Array.from({ length: count }).map((_, i) => (
      <ServerCardSkeleton key={i} />
    ))}
  </div>
);

export const PlayersTableLoading: React.FC<{ count?: number }> = ({ count = 5 }) => (
  <div className="space-y-2">
    {Array.from({ length: count }).map((_, i) => (
      <PlayerRowSkeleton key={i} />
    ))}
  </div>
);

export const StatsGridLoading: React.FC<{ count?: number }> = ({ count = 4 }) => (
  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4" data-testid="stats-grid-loading">
    {Array.from({ length: count }).map((_, i) => (
      <StatCardSkeleton key={i} />
    ))}
  </div>
);

export const ChartsLoading: React.FC<{ count?: number }> = ({ count = 2 }) => (
  <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
    {Array.from({ length: count }).map((_, i) => (
      <ChartSkeleton key={i} />
    ))}
  </div>
);

export const TableLoading: React.FC<{ 
  columns?: number; 
  rows?: number;
  className?: string;
}> = ({ columns = 4, rows = 5, className }) => (
  <div className={cn('space-y-2', className)}>
    {/* Header skeleton */}
    <div className="flex space-x-4 pb-2 border-b">
      {Array.from({ length: columns }).map((_, i) => (
        <div key={i} className="flex-1">
          <div className="h-4 w-3/4 bg-muted rounded animate-pulse" />
        </div>
      ))}
    </div>
    
    {/* Rows skeleton */}
    {Array.from({ length: rows }).map((_, i) => (
      <div key={i} className="flex space-x-4 py-2">
        {Array.from({ length: columns }).map((_, j) => (
          <div key={j} className="flex-1">
            <div className="h-4 w-full bg-muted rounded animate-pulse" />
          </div>
        ))}
      </div>
    ))}
  </div>
);

export const FormLoading: React.FC<{ fields?: number }> = ({ fields = 4 }) => (
  <div className="space-y-6">
    {Array.from({ length: fields }).map((_, i) => (
      <div key={i} className="space-y-2">
        <div className="h-4 w-24 bg-muted rounded animate-pulse" />
        <div className="h-10 w-full bg-muted rounded animate-pulse" />
      </div>
    ))}
    <div className="flex justify-end space-x-2">
      <div className="h-10 w-20 bg-muted rounded animate-pulse" />
      <div className="h-10 w-20 bg-muted rounded animate-pulse" />
    </div>
  </div>
);

export const SidebarLoading: React.FC = () => (
  <div className="space-y-4 p-4">
    <div className="space-y-2">
      <div className="h-4 w-16 bg-muted rounded animate-pulse" />
      <div className="h-8 w-full bg-muted rounded animate-pulse" />
    </div>
    
    <div className="space-y-2">
      <div className="h-4 w-20 bg-muted rounded animate-pulse" />
      {Array.from({ length: 3 }).map((_, i) => (
        <div key={i} className="flex items-center space-x-3 p-2">
          <div className="h-6 w-6 bg-muted rounded animate-pulse" />
          <div className="flex-1 space-y-1">
            <div className="h-4 w-24 bg-muted rounded animate-pulse" />
            <div className="h-3 w-16 bg-muted rounded animate-pulse" />
          </div>
        </div>
      ))}
    </div>
  </div>
);

export const HeaderLoading: React.FC = () => (
  <div className="flex items-center justify-between p-4 border-b">
    <div className="flex items-center space-x-4">
      <div className="h-6 w-32 bg-muted rounded animate-pulse" />
      <div className="h-6 w-16 bg-muted rounded animate-pulse" />
    </div>
    <div className="flex items-center space-x-2">
      <div className="h-8 w-20 bg-muted rounded animate-pulse" />
      <div className="h-8 w-20 bg-muted rounded animate-pulse" />
      <div className="h-8 w-20 bg-muted rounded animate-pulse" />
    </div>
  </div>
);

// Hook for managing loading states
export const useLoadingState = (initialState = false) => {
  const [isLoading, setIsLoading] = React.useState(initialState);
  const [error, setError] = React.useState<Error | null>(null);

  const startLoading = React.useCallback(() => {
    setIsLoading(true);
    setError(null);
  }, []);

  const stopLoading = React.useCallback(() => {
    setIsLoading(false);
  }, []);

  const setLoadingError = React.useCallback((error: Error) => {
    setIsLoading(false);
    setError(error);
  }, []);

  const reset = React.useCallback(() => {
    setIsLoading(false);
    setError(null);
  }, []);

  return {
    isLoading,
    error,
    startLoading,
    stopLoading,
    setLoadingError,
    reset,
  };
};
