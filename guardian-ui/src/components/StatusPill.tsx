import React from 'react';
import { Badge } from '@/components/ui/badge';
import { ServerStatus } from '@/lib/types';
import { Loader2, Play, Square, RotateCcw } from 'lucide-react';

interface StatusPillProps {
  status: ServerStatus;
  className?: string;
  showIcon?: boolean;
}

export const StatusPill: React.FC<StatusPillProps> = React.memo(({ 
  status, 
  className = '', 
  showIcon = true 
}) => {
  const getStatusConfig = (status: ServerStatus) => {
    switch (status) {
      case 'running':
        return {
          label: 'Running',
          className: 'status-running',
          icon: <Play className="h-3 w-3" />,
        };
      case 'stopped':
        return {
          label: 'Stopped',
          className: 'status-stopped',
          icon: <Square className="h-3 w-3" />,
        };
      case 'starting':
        return {
          label: 'Starting',
          className: 'status-starting',
          icon: <Loader2 className="h-3 w-3 animate-spin" />,
        };
      case 'stopping':
        return {
          label: 'Stopping',
          className: 'status-stopping',
          icon: <RotateCcw className="h-3 w-3 animate-spin" />,
        };
      default:
        return {
          label: 'Unknown',
          className: 'bg-gray-500/20 text-gray-400 border border-gray-500/30',
          icon: null,
        };
    }
  };

  const config = getStatusConfig(status);

  return (
    <Badge 
      variant="outline" 
      className={`${config.className} ${className} flex items-center gap-1`}
      data-testid="status-pill"
      role="status"
      aria-label={`Server status: ${status}`}
    >
      {showIcon && config.icon}
      {config.label}
    </Badge>
  );
});

export default StatusPill;
