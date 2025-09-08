import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { useServersStore } from '@/store/servers';
import { ErrorEmptyState } from '@/components/ui/EmptyState';
import WorldHeatmap from '@/components/World/WorldHeatmap';
import { LoadingWrapper } from '@/components/LoadingWrapper';
import { api } from '@/lib/api';

export const World: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchWorldData = async () => {
      if (!serverId) return;
      
      setIsLoading(true);
      setError(null);
      
      try {
        const response = await api.getWorldData(serverId);
        if (!response.ok) {
          setError(response.error || 'Failed to load world data');
        }
      } catch (err) {
        setError('Network error while loading world data');
      } finally {
        setIsLoading(false);
      }
    };

    fetchWorldData();
  }, [serverId]);

  if (!server) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view world data."
        />
      </div>
    );
  }

  if (error) {
    return (
      <LoadingWrapper
        isLoading={false}
        error={error}
        className="p-6"
      />
    );
  }

  if (isLoading) {
    return (
      <LoadingWrapper
        isLoading={true}
        error={null}
        className="p-6"
      />
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold">World Management</h2>
        <p className="text-muted-foreground">
          World visualization and management tools for {server.name}
        </p>
      </div>

      {/* World Heatmap Description */}
      <div className="panel p-6">
        <h3 className="text-lg font-semibold mb-3">World Activity Heatmap</h3>
        <div className="space-y-3 text-sm text-muted-foreground">
          <p>
            The heatmap below visualizes world activity and performance issues across your server's world. 
            This tool helps you identify problem areas and optimize your server's performance.
          </p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <h4 className="font-medium text-foreground mb-2">What the colors mean:</h4>
              <ul className="space-y-1">
                <li>• <span className="text-red-400">Red areas</span> - High activity or performance issues</li>
                <li>• <span className="text-orange-400">Orange areas</span> - Medium activity</li>
                <li>• <span className="text-gray-400">Gray areas</span> - Low activity or unloaded chunks</li>
              </ul>
            </div>
            <div>
              <h4 className="font-medium text-foreground mb-2">What to look for:</h4>
              <ul className="space-y-1">
                <li>• Concentrated red spots may indicate lag sources</li>
                <li>• Large red areas suggest heavy player activity</li>
                <li>• Use this data to optimize chunk loading and performance</li>
              </ul>
            </div>
          </div>
        </div>
      </div>

      {/* World Heatmap */}
      <WorldHeatmap />
    </div>
  );
};

export default World;