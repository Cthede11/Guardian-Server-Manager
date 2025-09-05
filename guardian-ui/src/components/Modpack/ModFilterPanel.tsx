import React from 'react';
import type { ModFilters, MinecraftVersion } from '../../lib/types/modpack';
import { MOD_SIDES, MOD_CATEGORIES, MOD_SOURCES } from '../../lib/types/modpack';

interface ModFilterPanelProps {
  filters: ModFilters;
  minecraftVersions: MinecraftVersion[];
  onFiltersChange: (filters: Partial<ModFilters>) => void;
  onClearFilters: () => void;
}

export const ModFilterPanel: React.FC<ModFilterPanelProps> = ({
  filters,
  minecraftVersions,
  onFiltersChange,
  onClearFilters
}) => {
  const handleVersionChange = (version: string) => {
    onFiltersChange({ minecraft_version: version });
  };

  const handleLoaderChange = (loader: ModFilters['loader']) => {
    onFiltersChange({ loader });
  };

  const handleCategoryChange = (category: ModFilters['category']) => {
    onFiltersChange({ category });
  };

  const handleSideChange = (side: ModFilters['side']) => {
    onFiltersChange({ side });
  };

  const handleSortChange = (sortBy: ModFilters['sort_by'], sortOrder: ModFilters['sort_order']) => {
    onFiltersChange({ sort_by: sortBy, sort_order: sortOrder });
  };

  const handleMinDownloadsChange = (minDownloads: number) => {
    onFiltersChange({ min_downloads: minDownloads });
  };

  const handleMaxFileSizeChange = (maxFileSize: number) => {
    onFiltersChange({ max_file_size: maxFileSize });
  };

  const handleClientVersionChange = (hasClientVersion: boolean) => {
    onFiltersChange({ has_client_version: hasClientVersion });
  };

  const handleServerVersionChange = (hasServerVersion: boolean) => {
    onFiltersChange({ has_server_version: hasServerVersion });
  };

  return (
    <div className="p-4 space-y-6">
      {/* Clear Filters */}
      <div className="flex justify-between items-center">
        <h3 className="text-sm font-medium text-gray-900">Filters</h3>
        <button
          onClick={onClearFilters}
          className="text-sm text-blue-600 hover:text-blue-700 font-medium"
        >
          Clear All
        </button>
      </div>

      {/* Minecraft Version */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Minecraft Version
        </label>
        <select
          value={filters.minecraft_version}
          onChange={(e) => handleVersionChange(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        >
          {minecraftVersions.map((version) => (
            <option key={version.id} value={version.id}>
              {version.id} ({version.release_type})
            </option>
          ))}
        </select>
      </div>

      {/* Mod Loader */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Mod Loader
        </label>
        <div className="space-y-2">
          <div className="flex items-center">
            <input
              type="radio"
              id="loader-forge"
              name="loader"
              value="forge"
              checked={filters.loader.type === 'forge'}
              onChange={() => handleLoaderChange({ type: 'forge', version: 'latest' })}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300"
            />
            <label htmlFor="loader-forge" className="ml-2 text-sm text-gray-700">
              Forge
            </label>
          </div>
          <div className="flex items-center">
            <input
              type="radio"
              id="loader-fabric"
              name="loader"
              value="fabric"
              checked={filters.loader.type === 'fabric'}
              onChange={() => handleLoaderChange({ type: 'fabric', version: 'latest' })}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300"
            />
            <label htmlFor="loader-fabric" className="ml-2 text-sm text-gray-700">
              Fabric
            </label>
          </div>
          <div className="flex items-center">
            <input
              type="radio"
              id="loader-quilt"
              name="loader"
              value="quilt"
              checked={filters.loader.type === 'quilt'}
              onChange={() => handleLoaderChange({ type: 'quilt', version: 'latest' })}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300"
            />
            <label htmlFor="loader-quilt" className="ml-2 text-sm text-gray-700">
              Quilt
            </label>
          </div>
          <div className="flex items-center">
            <input
              type="radio"
              id="loader-neoforge"
              name="loader"
              value="neoforge"
              checked={filters.loader.type === 'neoforge'}
              onChange={() => handleLoaderChange({ type: 'neoforge', version: 'latest' })}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300"
            />
            <label htmlFor="loader-neoforge" className="ml-2 text-sm text-gray-700">
              NeoForge
            </label>
          </div>
        </div>
      </div>

      {/* Category */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Category
        </label>
        <select
          value={filters.category}
          onChange={(e) => handleCategoryChange(e.target.value as ModFilters['category'])}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="all">All Categories</option>
          {MOD_CATEGORIES.map((category) => (
            <option key={category.type} value={category.type}>
              {category.icon} {category.label}
            </option>
          ))}
        </select>
      </div>

      {/* Side */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Side
        </label>
        <select
          value={filters.side}
          onChange={(e) => handleSideChange(e.target.value as ModFilters['side'])}
          className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="all">All Sides</option>
          {MOD_SIDES.map((side) => (
            <option key={side.type} value={side.type}>
              {side.icon} {side.label}
            </option>
          ))}
        </select>
      </div>

      {/* Sort By */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Sort By
        </label>
        <div className="space-y-2">
          <select
            value={filters.sort_by}
            onChange={(e) => handleSortChange(e.target.value as ModFilters['sort_by'], filters.sort_order)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="name">Name</option>
            <option value="popularity">Popularity</option>
            <option value="downloads">Downloads</option>
            <option value="updated">Last Updated</option>
            <option value="version">Version</option>
          </select>
          <select
            value={filters.sort_order}
            onChange={(e) => handleSortChange(filters.sort_by, e.target.value as ModFilters['sort_order'])}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="asc">Ascending</option>
            <option value="desc">Descending</option>
          </select>
        </div>
      </div>

      {/* Advanced Filters */}
      <div className="border-t border-gray-200 pt-4">
        <h4 className="text-sm font-medium text-gray-900 mb-3">Advanced Filters</h4>
        
        {/* Min Downloads */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Minimum Downloads
          </label>
          <input
            type="number"
            value={filters.min_downloads || ''}
            onChange={(e) => handleMinDownloadsChange(parseInt(e.target.value) || 0)}
            placeholder="0"
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        {/* Max File Size */}
        <div className="mb-4">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Max File Size (MB)
          </label>
          <input
            type="number"
            value={filters.max_file_size ? filters.max_file_size / (1024 * 1024) : ''}
            onChange={(e) => handleMaxFileSizeChange((parseInt(e.target.value) || 0) * 1024 * 1024)}
            placeholder="No limit"
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        {/* Version Requirements */}
        <div className="space-y-2">
          <div className="flex items-center">
            <input
              type="checkbox"
              id="has-client-version"
              checked={filters.has_client_version || false}
              onChange={(e) => handleClientVersionChange(e.target.checked)}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <label htmlFor="has-client-version" className="ml-2 text-sm text-gray-700">
              Has Client Version
            </label>
          </div>
          <div className="flex items-center">
            <input
              type="checkbox"
              id="has-server-version"
              checked={filters.has_server_version || false}
              onChange={(e) => handleServerVersionChange(e.target.checked)}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <label htmlFor="has-server-version" className="ml-2 text-sm text-gray-700">
              Has Server Version
            </label>
          </div>
        </div>
      </div>
    </div>
  );
};
