import React from 'react';
import { Button } from './button';
import { cn } from '@/lib/utils';
import { 
  Server, 
  Users, 
  FileText, 
  BarChart3, 
  Settings, 
  AlertCircle,
  Plus,
  RefreshCw,
  Search
} from 'lucide-react';

interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  description?: string;
  action?: {
    label: string;
    onClick: () => void;
    variant?: 'default' | 'outline' | 'secondary' | 'ghost' | 'link' | 'destructive';
  };
  secondaryAction?: {
    label: string;
    onClick: () => void;
    variant?: 'default' | 'outline' | 'secondary' | 'ghost' | 'link' | 'destructive';
  };
  className?: string;
  size?: 'sm' | 'md' | 'lg';
}

const defaultIcons = {
  servers: <Server className="h-12 w-12 text-muted-foreground" />,
  players: <Users className="h-12 w-12 text-muted-foreground" />,
  files: <FileText className="h-12 w-12 text-muted-foreground" />,
  charts: <BarChart3 className="h-12 w-12 text-muted-foreground" />,
  settings: <Settings className="h-12 w-12 text-muted-foreground" />,
  error: <AlertCircle className="h-12 w-12 text-destructive" />,
  search: <Search className="h-12 w-12 text-muted-foreground" />,
  add: <Plus className="h-12 w-12 text-muted-foreground" />,
  refresh: <RefreshCw className="h-12 w-12 text-muted-foreground" />,
};

export const EmptyState: React.FC<EmptyStateProps> = React.memo(({
  icon,
  title,
  description,
  action,
  secondaryAction,
  className,
  size = 'md',
}) => {
  const sizeClasses = {
    sm: 'py-8',
    md: 'py-12',
    lg: 'py-16',
  };

  const iconSizeClasses = {
    sm: 'h-8 w-8',
    md: 'h-12 w-12',
    lg: 'h-16 w-16',
  };

  return (
    <div className={cn(
      'flex flex-col items-center justify-center text-center',
      sizeClasses[size],
      className
    )}>
      {icon && (
        <div className={cn('mb-4', iconSizeClasses[size])}>
          {icon}
        </div>
      )}
      
      <h3 className={cn(
        'font-semibold text-foreground mb-2',
        size === 'sm' ? 'text-lg' : size === 'lg' ? 'text-2xl' : 'text-xl'
      )}>
        {title}
      </h3>
      
      {description && (
        <p className={cn(
          'text-muted-foreground mb-6 max-w-md',
          size === 'sm' ? 'text-sm' : 'text-base'
        )}>
          {description}
        </p>
      )}
      
      {(action || secondaryAction) && (
        <div className="flex flex-col sm:flex-row gap-3">
          {action && (
            <Button
              onClick={action.onClick}
              variant={action.variant || 'default'}
              size={size === 'sm' ? 'sm' : 'default'}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault()
                  action.onClick()
                }
              }}
            >
              {action.label}
            </Button>
          )}
          {secondaryAction && (
            <Button
              onClick={secondaryAction.onClick}
              variant={secondaryAction.variant || 'outline'}
              size={size === 'sm' ? 'sm' : 'default'}
            >
              {secondaryAction.label}
            </Button>
          )}
        </div>
      )}
    </div>
  );
});

// Predefined empty states for common scenarios
export const NoServersEmptyState: React.FC<{ onCreateServer: () => void }> = ({ onCreateServer }) => (
  <EmptyState
    icon={defaultIcons.servers}
    title="No servers found"
    description="Get started by creating your first Minecraft server. You can add multiple servers and manage them all from this dashboard."
    action={{
      label: 'Create Server',
      onClick: onCreateServer,
    }}
  />
);

export const NoPlayersEmptyState: React.FC<{ onRefresh: () => void }> = ({ onRefresh }) => (
  <EmptyState
    icon={defaultIcons.players}
    title="No players online"
    description="The server is running but no players are currently connected. Players will appear here when they join."
    secondaryAction={{
      label: 'Refresh',
      onClick: onRefresh,
      variant: 'outline',
    }}
  />
);

export const NoDataEmptyState: React.FC<{ 
  title: string; 
  description?: string; 
  onRefresh?: () => void;
  onAdd?: () => void;
}> = ({ title, description, onRefresh, onAdd }) => (
  <EmptyState
    icon={defaultIcons.files}
    title={title}
    description={description || "No data available. Try refreshing or check your server status."}
    action={onAdd ? {
      label: 'Add New',
      onClick: onAdd,
    } : undefined}
    secondaryAction={onRefresh ? {
      label: 'Refresh',
      onClick: onRefresh,
      variant: 'outline',
    } : undefined}
  />
);

export const SearchEmptyState: React.FC<{ 
  query: string; 
  onClear: () => void;
}> = ({ query, onClear }) => (
  <EmptyState
    icon={defaultIcons.search}
    title="No results found"
    description={`No items match your search for "${query}". Try adjusting your search terms.`}
    secondaryAction={{
      label: 'Clear Search',
      onClick: onClear,
      variant: 'outline',
    }}
  />
);

export const ErrorEmptyState: React.FC<{ 
  title?: string;
  description?: string; 
  onRetry?: () => void;
}> = ({ 
  title = "Something went wrong", 
  description = "An error occurred while loading data. Please try again.",
  onRetry 
}) => (
  <EmptyState
    icon={defaultIcons.error}
    title={title}
    description={description}
    action={onRetry ? {
      label: 'Try Again',
      onClick: onRetry,
    } : undefined}
  />
);

export const LoadingEmptyState: React.FC<{ message?: string }> = ({ 
  message = "Loading..." 
}) => (
  <EmptyState
    icon={<RefreshCw className="h-12 w-12 text-muted-foreground animate-spin" />}
    title={message}
    size="sm"
  />
);
