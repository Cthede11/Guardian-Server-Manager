import React from 'react';
import { ConsoleStream } from '@/components/Console/ConsoleStream';
import { PlayersTable } from '@/components/Tables/PlayersTable';
import { World as WorldPage } from './World';
import { ModsRules as ModsRulesPage } from './ModsRules';
import { Performance as PerformancePage } from './Performance';
import { Backups as BackupsPage } from './Backups';
import { Events as EventsPage } from './Events';
import { Pregen as PregenPage } from './Pregen';
import { Sharding as ShardingPage } from './Sharding';
import { Diagnostics as DiagnosticsPage } from './Diagnostics';
import { Overview as OverviewPage } from './Overview';
import * as SettingsPages from './Settings';

// Overview page
export const Overview: React.FC = () => {
  return (
    <div className="h-full">
      <OverviewPage />
    </div>
  );
};

// Console page
export const Console: React.FC = () => {
  return (
    <div className="h-full">
      <ConsoleStream />
    </div>
  );
};

// Players page
export const Players: React.FC = () => {
  return (
    <div className="h-full">
      <PlayersTable />
    </div>
  );
};

// World page
export const World: React.FC = () => {
  return (
    <div className="h-full">
      <WorldPage />
    </div>
  );
};

// Mods & Rules page
export const ModsRules: React.FC = () => {
  return (
    <div className="h-full">
      <ModsRulesPage />
    </div>
  );
};

// Performance page
export const Performance: React.FC = () => {
  return (
    <div className="h-full">
      <PerformancePage />
    </div>
  );
};

// Backups page
export const Backups: React.FC = () => {
  return (
    <div className="h-full">
      <BackupsPage />
    </div>
  );
};

// Events page
export const Events: React.FC = () => {
  return (
    <div className="h-full">
      <EventsPage />
    </div>
  );
};

// Pregen page
export const Pregen: React.FC = () => {
  return (
    <div className="h-full">
      <PregenPage />
    </div>
  );
};

// Sharding page
export const Sharding: React.FC = () => {
  return (
    <div className="h-full">
      <ShardingPage />
    </div>
  );
};

// Diagnostics page
export const Diagnostics: React.FC = () => {
  return (
    <div className="h-full">
      <DiagnosticsPage />
    </div>
  );
};

// Settings pages
export const Settings = SettingsPages;
