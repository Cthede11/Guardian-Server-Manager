import React from 'react';
import { PlayersTable } from '@/components/Tables/PlayersTable';

export const Players: React.FC = () => {
  return (
    <div className="h-full">
      <PlayersTable />
    </div>
  );
};

export default Players;
