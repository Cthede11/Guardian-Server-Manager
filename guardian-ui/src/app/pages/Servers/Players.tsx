import React from 'react';
import { useParams } from 'react-router-dom';
import { useServersStore } from '@/store/servers';
import { PlayersTable } from '@/components/Tables/PlayersTable';
import { ErrorEmptyState } from '@/components/ui/EmptyState';

export const Players: React.FC = () => {
  const { id: serverId } = useParams<{ id: string }>();
  const { getServerById } = useServersStore();
  const server = serverId ? getServerById(serverId) : null;

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
