import React from 'react';
import type { Modpack } from '../../lib/types/modpack';

interface ModpackGridProps {
  modpacks: Modpack[];
  loading: boolean;
  onModpackSelect: (modpack: Modpack) => void;
}

export const ModpackGrid: React.FC<ModpackGridProps> = ({ modpacks, loading, onModpackSelect }) => {
  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} className="bg-white rounded-lg border border-gray-200 p-6 animate-pulse">
            <div className="h-6 bg-gray-200 rounded mb-3"></div>
            <div className="h-4 bg-gray-200 rounded mb-4 w-3/4"></div>
            <div className="space-y-2">
              <div className="h-3 bg-gray-200 rounded"></div>
              <div className="h-3 bg-gray-200 rounded w-5/6"></div>
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (modpacks.length === 0) {
    return (
      <div className="text-center py-12">
        <svg
          className="mx-auto h-12 w-12 text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
          />
        </svg>
        <h3 className="mt-2 text-sm font-medium text-gray-900">No modpacks found</h3>
        <p className="mt-1 text-sm text-gray-500">
          Create your first modpack to get started.
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      {modpacks.map((modpack) => (
        <div
          key={modpack.id}
          onClick={() => onModpackSelect(modpack)}
          className="bg-white rounded-lg border border-gray-200 p-6 hover:shadow-lg transition-shadow cursor-pointer group"
        >
          <div className="flex items-start justify-between mb-4">
            <h3 className="text-lg font-semibold text-gray-900 group-hover:text-blue-600 transition-colors">
              {modpack.name}
            </h3>
            <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
              {modpack.loader}
            </span>
          </div>

          {modpack.description && (
            <p className="text-sm text-gray-600 mb-4 line-clamp-2">
              {modpack.description}
            </p>
          )}

          <div className="space-y-2 mb-4">
            <div className="flex items-center text-sm text-gray-500">
              <span className="font-medium">Minecraft:</span>
              <span className="ml-2 px-2 py-1 bg-green-100 text-green-800 rounded text-xs">
                {modpack.minecraft_version}
              </span>
            </div>
            <div className="flex items-center text-sm text-gray-500">
              <span className="font-medium">Client Mods:</span>
              <span className="ml-2">{modpack.client_mods.length}</span>
            </div>
            <div className="flex items-center text-sm text-gray-500">
              <span className="font-medium">Server Mods:</span>
              <span className="ml-2">{modpack.server_mods.length}</span>
            </div>
          </div>

          <div className="flex items-center justify-between text-xs text-gray-400">
            <span>Created {new Date(modpack.created_at).toLocaleDateString()}</span>
            <span>Updated {new Date(modpack.updated_at).toLocaleDateString()}</span>
          </div>
        </div>
      ))}
    </div>
  );
};
