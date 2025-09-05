import React from 'react';
import type { ModInfo } from '../../lib/types/modpack';
import { MOD_SIDES, MOD_CATEGORIES, MOD_SOURCES } from '../../lib/types/modpack';

interface ModGridProps {
  mods: ModInfo[];
  loading: boolean;
  onModSelect: (mod: ModInfo) => void;
}

export const ModGrid: React.FC<ModGridProps> = ({ mods, loading, onModSelect }) => {
  const getSideInfo = (side: ModInfo['side']) => {
    return MOD_SIDES.find(s => s.type === side) || MOD_SIDES[MOD_SIDES.length - 1];
  };

  const getCategoryInfo = (category: ModInfo['category']) => {
    return MOD_CATEGORIES.find(c => c.type === category) || MOD_CATEGORIES[MOD_CATEGORIES.length - 1];
  };

  const getSourceInfo = (source: ModInfo['source']) => {
    return MOD_SOURCES.find(s => s.type === source) || MOD_SOURCES[MOD_SOURCES.length - 1];
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDownloadCount = (count?: number) => {
    if (!count) return 'N/A';
    if (count >= 1000000) return (count / 1000000).toFixed(1) + 'M';
    if (count >= 1000) return (count / 1000).toFixed(1) + 'K';
    return count.toString();
  };

  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {Array.from({ length: 8 }).map((_, i) => (
          <div key={i} className="bg-white rounded-lg border border-gray-200 p-4 animate-pulse">
            <div className="h-4 bg-gray-200 rounded mb-2"></div>
            <div className="h-3 bg-gray-200 rounded mb-4 w-3/4"></div>
            <div className="flex space-x-2 mb-4">
              <div className="h-6 bg-gray-200 rounded w-16"></div>
              <div className="h-6 bg-gray-200 rounded w-20"></div>
            </div>
            <div className="h-3 bg-gray-200 rounded w-1/2"></div>
          </div>
        ))}
      </div>
    );
  }

  if (mods.length === 0) {
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
            d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
          />
        </svg>
        <h3 className="mt-2 text-sm font-medium text-gray-900">No mods found</h3>
        <p className="mt-1 text-sm text-gray-500">
          Try adjusting your search criteria or filters.
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
      {mods.map((mod) => {
        const sideInfo = getSideInfo(mod.side);
        const categoryInfo = getCategoryInfo(mod.category);
        const sourceInfo = getSourceInfo(mod.source);

        return (
          <div
            key={mod.id}
            onClick={() => onModSelect(mod)}
            className="bg-white rounded-lg border border-gray-200 p-4 hover:shadow-lg transition-shadow cursor-pointer group"
          >
            {/* Header */}
            <div className="flex items-start justify-between mb-3">
              <h3 className="text-lg font-semibold text-gray-900 group-hover:text-blue-600 transition-colors">
                {mod.name}
              </h3>
              <div className="flex items-center space-x-1">
                <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-${sideInfo.color}-100 text-${sideInfo.color}-800`}>
                  {sideInfo.icon}
                </span>
                <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-${categoryInfo.color}-100 text-${categoryInfo.color}-800`}>
                  {categoryInfo.icon}
                </span>
              </div>
            </div>

            {/* Description */}
            {mod.description && (
              <p className="text-sm text-gray-600 mb-3 line-clamp-2">
                {mod.description}
              </p>
            )}

            {/* Version and Loader */}
            <div className="flex items-center space-x-2 mb-3">
              <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-gray-100 text-gray-800">
                v{mod.version}
              </span>
              <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-blue-100 text-blue-800">
                {mod.loader_versions && Object.keys(mod.loader_versions).length > 0 
                  ? Object.keys(mod.loader_versions)[0] 
                  : 'Unknown'
                }
              </span>
            </div>

            {/* Minecraft Versions */}
            <div className="mb-3">
              <div className="flex flex-wrap gap-1">
                {mod.minecraft_versions.slice(0, 3).map((version) => (
                  <span
                    key={version}
                    className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-green-100 text-green-800"
                  >
                    {version}
                  </span>
                ))}
                {mod.minecraft_versions.length > 3 && (
                  <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-gray-100 text-gray-800">
                    +{mod.minecraft_versions.length - 3} more
                  </span>
                )}
              </div>
            </div>

            {/* Stats */}
            <div className="flex items-center justify-between text-sm text-gray-500">
              <div className="flex items-center space-x-4">
                <span className="flex items-center">
                  <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
                  </svg>
                  {formatDownloadCount(mod.download_count)}
                </span>
                <span className="flex items-center">
                  <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                  </svg>
                  {mod.rating ? mod.rating.toFixed(1) : 'N/A'}
                </span>
              </div>
              <span className="text-xs">
                {formatFileSize(mod.file_size)}
              </span>
            </div>

            {/* Tags */}
            {mod.tags.length > 0 && (
              <div className="mt-3 flex flex-wrap gap-1">
                {mod.tags.slice(0, 3).map((tag) => (
                  <span
                    key={tag}
                    className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-gray-100 text-gray-700"
                  >
                    #{tag}
                  </span>
                ))}
                {mod.tags.length > 3 && (
                  <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-gray-100 text-gray-700">
                    +{mod.tags.length - 3}
                  </span>
                )}
              </div>
            )}

            {/* Source */}
            <div className="mt-3 flex items-center justify-between">
              <div className="flex items-center">
                <span className={`inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-${sourceInfo.color}-100 text-${sourceInfo.color}-800`}>
                  {sourceInfo.icon} {sourceInfo.label}
                </span>
              </div>
              <span className="text-xs text-gray-400">
                {new Date(mod.last_updated).toLocaleDateString()}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
};
