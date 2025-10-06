// import React from 'react';
import { createBrowserRouter, Navigate } from 'react-router-dom';
import AppShell from './layout/AppShell';
import { ServersPages, WorkspacePages } from './pages';
import Console from './pages/Console';
import ModpackBrowser from '../components/Modpack/ModpackBrowser';
import { ServerListDashboard } from '../components/Dashboard/ServerListDashboard';
import { ErrorBoundary } from '../components/ErrorBoundary';

export const router = createBrowserRouter([
  {
    path: '/',
    element: <AppShell />,
    children: [
      {
        index: true,
        element: <Navigate to="/servers" replace />,
      },
      {
        path: 'servers',
        children: [
          {
            index: true,
            element: <ServerListDashboard />,
          },
          {
            path: ':id',
            children: [
              {
                index: true,
                element: <Navigate to="overview" replace />,
              },
              {
                path: 'overview',
                element: <ServersPages.Overview />,
              },
              {
                path: 'console',
                element: <ServersPages.Console />,
              },
              {
                path: 'players',
                element: <ServersPages.Players />,
              },
              {
                path: 'world',
                element: <ServersPages.World />,
              },
              {
                path: 'mods-rules',
                element: <ServersPages.ModsRules />,
              },
              {
                path: 'performance',
                element: <ServersPages.Performance />,
              },
              {
                path: 'backups',
                element: <ServersPages.Backups />,
              },
              {
                path: 'events',
                element: <ServersPages.Events />,
              },
              {
                path: 'pregen',
                element: <ServersPages.Pregen />,
              },
              {
                path: 'sharding',
                element: <ServersPages.Sharding />,
              },
              {
                path: 'diagnostics',
                element: <ServersPages.Diagnostics />,
              },
              {
                path: 'settings',
                children: [
                  {
                    index: true,
                    element: <Navigate to="general" replace />,
                  },
                  {
                    path: 'general',
                    element: <ServersPages.Settings.General />,
                  },
                  {
                    path: 'jvm',
                    element: <ServersPages.Settings.JVM />,
                  },
                  {
                    path: 'gpu',
                    element: <ServersPages.Settings.GPU />,
                  },
                  {
                    path: 'ha',
                    element: <ServersPages.Settings.HA />,
                  },
                  {
                    path: 'paths',
                    element: <ServersPages.Settings.Paths />,
                  },
                  {
                    path: 'composer',
                    element: <ServersPages.Settings.Composer />,
                  },
                  {
                    path: 'tokens',
                    element: <ServersPages.Settings.Tokens />,
                  },
                ],
              },
            ],
          },
        ],
      },
      {
        path: 'console',
        element: <Console />,
      },
      {
        path: 'modpacks',
        element: (
          <ErrorBoundary>
            <ModpackBrowser />
          </ErrorBoundary>
        ),
      },
      {
        path: 'workspace',
        children: [
          {
            index: true,
            element: <Navigate to="/workspace/users-roles" replace />,
          },
          {
            path: 'users-roles',
            element: <WorkspacePages.UsersRoles />,
          },
          {
            path: 'backup-targets',
            element: <WorkspacePages.BackupTargets />,
          },
          {
            path: 'tokens',
            element: <WorkspacePages.Tokens />,
          },
          {
            path: 'theme',
            element: <WorkspacePages.Theme />,
          },
        ],
      },
    ],
  },
]);
