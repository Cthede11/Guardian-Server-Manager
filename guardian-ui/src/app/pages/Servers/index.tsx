import React, { Suspense, lazy } from 'react';
import { Skeleton } from '@/components/ui/skeleton';
import { ErrorBoundary } from '@/components/ErrorBoundary';

// Lazy load all server pages for code splitting
const OverviewPage = lazy(() => import('./Overview').then(m => ({ default: m.Overview })));
const ConsolePage = lazy(() => import('./Console').then(m => ({ default: m.Console })));
const PlayersPage = lazy(() => import('./Players').then(m => ({ default: m.Players })));
const WorldPage = lazy(() => import('./World').then(m => ({ default: m.World })));
const ModsRulesPage = lazy(() => import('./ModsRules').then(m => ({ default: m.ModsRules })));
const PerformancePage = lazy(() => import('./Performance').then(m => ({ default: m.Performance })));
const BackupsPage = lazy(() => import('./Backups').then(m => ({ default: m.Backups })));
const EventsPage = lazy(() => import('./Events').then(m => ({ default: m.Events })));
const PregenPage = lazy(() => import('./Pregen').then(m => ({ default: m.Pregen })));
const ShardingPage = lazy(() => import('./Sharding').then(m => ({ default: m.Sharding })));
const DiagnosticsPage = lazy(() => import('./Diagnostics').then(m => ({ default: m.Diagnostics })));

// Lazy load settings pages - all use the same Settings component
const GeneralSettingsPage = lazy(() => import('./Settings').then(m => ({ default: m.Settings })));
const JVMSettingsPage = lazy(() => import('./Settings').then(m => ({ default: m.Settings })));
const GPUSettingsPage = lazy(() => import('./Settings').then(m => ({ default: m.Settings })));
const HASettingsPage = lazy(() => import('./Settings').then(m => ({ default: m.Settings })));
const PathsSettingsPage = lazy(() => import('./Settings').then(m => ({ default: m.Settings })));
const ComposerSettingsPage = lazy(() => import('./Settings').then(m => ({ default: m.Settings })));
const TokensSettingsPage = lazy(() => import('./Settings').then(m => ({ default: m.Settings })));

// Skeleton component for loading states
const PageSkeleton: React.FC = () => (
  <div className="h-full space-y-4 p-6">
    <Skeleton className="h-8 w-64" />
    <div className="grid gap-4">
      <Skeleton className="h-32 w-full" />
      <Skeleton className="h-32 w-full" />
      <Skeleton className="h-32 w-full" />
    </div>
  </div>
);

// Overview page
export const Overview: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <OverviewPage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Console page
export const Console: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <ConsolePage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Players page
export const Players: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <PlayersPage />
      </Suspense>
    </ErrorBoundary>
  );
};

// World page
export const World: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <WorldPage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Mods & Rules page
export const ModsRules: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <ModsRulesPage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Performance page
export const Performance: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <PerformancePage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Backups page
export const Backups: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <BackupsPage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Events page
export const Events: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <EventsPage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Pregen page
export const Pregen: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <PregenPage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Sharding page
export const Sharding: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <ShardingPage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Diagnostics page
export const Diagnostics: React.FC = () => {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <DiagnosticsPage />
      </Suspense>
    </ErrorBoundary>
  );
};

// Settings pages with lazy loading
export const Settings = {
  General: () => (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <GeneralSettingsPage />
      </Suspense>
    </ErrorBoundary>
  ),
  JVM: () => (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <JVMSettingsPage />
      </Suspense>
    </ErrorBoundary>
  ),
  GPU: () => (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <GPUSettingsPage />
      </Suspense>
    </ErrorBoundary>
  ),
  HA: () => (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <HASettingsPage />
      </Suspense>
    </ErrorBoundary>
  ),
  Paths: () => (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <PathsSettingsPage />
      </Suspense>
    </ErrorBoundary>
  ),
  Composer: () => (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <ComposerSettingsPage />
      </Suspense>
    </ErrorBoundary>
  ),
  Tokens: () => (
    <ErrorBoundary>
      <Suspense fallback={<PageSkeleton />}>
        <TokensSettingsPage />
      </Suspense>
    </ErrorBoundary>
  ),
};
