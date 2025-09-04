import React from 'react';
import { ConsoleStream } from '@/components/Console/ConsoleStream';

export const Console: React.FC = () => {
  return (
    <div className="h-full">
      <ConsoleStream />
    </div>
  );
};

export default Console;
