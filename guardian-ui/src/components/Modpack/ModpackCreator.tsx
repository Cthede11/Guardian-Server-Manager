import React, { useState } from 'react';
import { useModpackStore } from '../../store/modpack';
import type { Modpack, ModInfo } from '../../lib/types/modpack';

export const ModpackCreator: React.FC = () => {
  const { createModpack, loading } = useModpackStore();
  const [modpack, setModpack] = useState<Partial<Modpack>>({
    name: '',
    description: '',
    minecraft_version: '1.21.1',
    loader: { type: 'forge', version: '1.20.1' },
    client_mods: [],
    server_mods: [],
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (modpack.name && modpack.minecraft_version && modpack.loader) {
      await createModpack(modpack as Omit<Modpack, 'id' | 'created_at' | 'updated_at'>);
    }
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-6">Create New Modpack</h2>
        
        <form onSubmit={handleSubmit} className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Modpack Name
            </label>
            <input
              type="text"
              value={modpack.name || ''}
              onChange={(e) => setModpack({ ...modpack, name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="Enter modpack name"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Description
            </label>
            <textarea
              value={modpack.description || ''}
              onChange={(e) => setModpack({ ...modpack, description: e.target.value })}
              rows={3}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="Enter modpack description"
            />
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Minecraft Version
              </label>
              <input
                type="text"
                value={modpack.minecraft_version || ''}
                onChange={(e) => setModpack({ ...modpack, minecraft_version: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="1.21.1"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Mod Loader
              </label>
              <select
                value={modpack.loader?.type || 'forge'}
                onChange={(e) => setModpack({ 
                  ...modpack, 
                  loader: { type: e.target.value as any, version: '1.20.1' }
                })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="forge">Forge</option>
                <option value="fabric">Fabric</option>
                <option value="quilt">Quilt</option>
                <option value="neoforge">NeoForge</option>
              </select>
            </div>
          </div>

          <div className="flex justify-end space-x-4">
            <button
              type="button"
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? 'Creating...' : 'Create Modpack'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
