import React, { useState, useEffect } from 'react';
import { useModpackStore } from '../../store/modpack';
import type { ModFilters, ModSide, ModCategory, ModSource } from '../../lib/types/modpack';
import { ModFilterPanel } from './ModFilterPanel';
import { ModSearchBar } from './ModSearchBar';
import { ModGrid } from './ModGrid';
import { ModpackGrid } from './ModpackGrid';
import { ModpackCreator } from './ModpackCreator';
import { CompatibilityChecker } from './CompatibilityChecker';

const ModpackBrowser: React.FC = () => {
  const {
    modpacks,
    mods,
    minecraftVersions,
    loading,
    error,
    filters,
    loadModpacks,
    loadMods,
    loadMinecraftVersions,
    searchMods,
    setFilters,
    clearFilters
  } = useModpackStore();

  const [activeTab, setActiveTab] = useState<'mods' | 'modpacks' | 'create' | 'compatibility'>('mods');
  const [showFilters, setShowFilters] = useState(true);

  useEffect(() => {
    loadMinecraftVersions();
    loadModpacks();
    loadMods();
  }, []);

  const handleSearch = (query: string) => {
    searchMods({ search_query: query });
  };

  const handleFiltersChange = (newFilters: Partial<ModFilters>) => {
    setFilters(newFilters);
    searchMods(newFilters);
  };

  const handleClearFilters = () => {
    clearFilters();
    loadMods();
  };

  if (loading && mods.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="flex">
          <div className="flex-shrink-0">
            <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
            </svg>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-red-800">Error loading modpack data</h3>
            <div className="mt-2 text-sm text-red-700">
              <p>{error}</p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full bg-gradient-to-br from-background via-background to-muted/20">
      {/* Sidebar */}
      <div className={`${showFilters ? 'w-80' : 'w-16'} transition-all duration-300 bg-card/95 backdrop-blur-sm border-r border-border/50 flex flex-col shadow-xl`}>
        <div className="p-4 border-b border-border/50 bg-gradient-to-r from-primary/10 to-secondary/10">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-foreground bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
              {showFilters ? 'Modpack Manager' : 'MM'}
            </h2>
            <button
              onClick={() => setShowFilters(!showFilters)}
              className="p-2 rounded-lg hover:bg-primary/20 transition-all duration-200 hover:scale-105"
            >
              <svg className="w-5 h-5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
          </div>
        </div>

        {showFilters && (
          <div className="flex-1 overflow-y-auto">
            <ModFilterPanel
              filters={filters}
              minecraftVersions={minecraftVersions}
              onFiltersChange={handleFiltersChange}
              onClearFilters={handleClearFilters}
            />
          </div>
        )}
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <div className="bg-card/80 backdrop-blur-sm border-b border-border/50 p-4 shadow-lg">
          <div className="flex items-center justify-between">
            <div className="flex space-x-1">
              <button
                onClick={() => setActiveTab('mods')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 hover:scale-105 ${
                  activeTab === 'mods'
                    ? 'bg-gradient-to-r from-primary to-secondary text-primary-foreground shadow-lg shadow-primary/25'
                    : 'text-muted-foreground hover:text-foreground hover:bg-gradient-to-r hover:from-primary/10 hover:to-secondary/10'
                }`}
              >
                Mods
              </button>
              <button
                onClick={() => setActiveTab('modpacks')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 hover:scale-105 ${
                  activeTab === 'modpacks'
                    ? 'bg-gradient-to-r from-primary to-secondary text-primary-foreground shadow-lg shadow-primary/25'
                    : 'text-muted-foreground hover:text-foreground hover:bg-gradient-to-r hover:from-primary/10 hover:to-secondary/10'
                }`}
              >
                Modpacks
              </button>
              <button
                onClick={() => setActiveTab('create')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 hover:scale-105 ${
                  activeTab === 'create'
                    ? 'bg-gradient-to-r from-primary to-secondary text-primary-foreground shadow-lg shadow-primary/25'
                    : 'text-muted-foreground hover:text-foreground hover:bg-gradient-to-r hover:from-primary/10 hover:to-secondary/10'
                }`}
              >
                Create
              </button>
              <button
                onClick={() => setActiveTab('compatibility')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 hover:scale-105 ${
                  activeTab === 'compatibility'
                    ? 'bg-gradient-to-r from-primary to-secondary text-primary-foreground shadow-lg shadow-primary/25'
                    : 'text-muted-foreground hover:text-foreground hover:bg-gradient-to-r hover:from-primary/10 hover:to-secondary/10'
                }`}
              >
                Compatibility
              </button>
            </div>

            <ModSearchBar
              onSearch={handleSearch}
              placeholder="Search mods..."
            />
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 bg-gradient-to-br from-background via-background to-muted/5">
          {activeTab === 'mods' && (
            <ModGrid
              mods={mods}
              loading={loading}
              onModSelect={(mod) => {
                // Handle mod selection
                console.log('Selected mod:', mod);
              }}
            />
          )}

          {activeTab === 'modpacks' && (
            <ModpackGrid
              modpacks={modpacks}
              loading={loading}
              onModpackSelect={(modpack) => {
                // Handle modpack selection
                console.log('Selected modpack:', modpack);
              }}
            />
          )}

          {activeTab === 'create' && (
            <ModpackCreator />
          )}

          {activeTab === 'compatibility' && (
            <CompatibilityChecker />
          )}
        </div>
      </div>
    </div>
  );
};

export default ModpackBrowser;
