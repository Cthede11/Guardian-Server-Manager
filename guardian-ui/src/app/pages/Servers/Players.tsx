import React from 'react';
import { useParams } from 'react-router-dom';
import { useServers } from '@/store/servers-new';
import { useServerStreams } from '@/app/hooks/useServerStreams';
import { PlayersTable } from '@/components/Tables/PlayersTable';
import { ErrorEmptyState } from '@/components/ui/EmptyState';

export const Players: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById, select } = useServers();
  const server = serverId ? getServerById(serverId) : null;

  // Select the server when the component mounts
  React.useEffect(() => {
    if (serverId) {
      select(serverId);
    }
  }, [serverId, select]);

  // Attach streams for the selected server
  useServerStreams(serverId);

  if (!server) {
    return (
      <div className="p-6">
        <ErrorEmptyState
          title="No server selected"
          description="Please select a server from the sidebar to view its players."
        />
      </div>
    );
  }

  return (
    <div className="h-full">
      <PlayersTable />
    </div>
  );
};

export default Players;
