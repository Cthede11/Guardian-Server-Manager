import React from 'react';
import { useParams } from 'react-router-dom';
import { useServersStore } from '@/store/servers';
import WorldHeatmap from '@/components/World/WorldHeatmap';

export const World: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;

  if (!server) {
    return (
      <div className="p-6">
        <div className="text-center py-12">
          <p className="text-muted-foreground">Select a server to view world data</p>
        </div>
      </div>
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

      {/* World Heatmap */}
      <WorldHeatmap />
    </div>
  );
};

export default World;