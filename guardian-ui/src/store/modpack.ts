import { create } from 'zustand';
import { modpackApi } from '../lib/api/modpack';
import { MINECRAFT_VERSIONS, getVersionsForModpack } from '../lib/constants/minecraft-versions';
import type { 
  Modpack, 
  ModInfo, 
  MinecraftVersion, 
  ModpackCompatibility, 
  ModFilters, 
  ModSearchResult 
} from '../lib/types/modpack';

interface ModpackState {
  // Data
  modpacks: Modpack[];
  mods: ModInfo[];
  minecraftVersions: MinecraftVersion[];
  searchResults: ModSearchResult | null;
  
  // UI State
  loading: boolean;
  error: string | null;
  selectedModpack: Modpack | null;
  selectedMod: ModInfo | null;
  
  // Filters
  filters: ModFilters;
  
  // Actions
  loadModpacks: () => Promise<void>;
  loadMods: (filters?: Partial<ModFilters>) => Promise<void>;
  loadMinecraftVersions: () => Promise<void>;
  searchMods: (filters: Partial<ModFilters>) => Promise<void>;
  
  createModpack: (modpack: Omit<Modpack, 'id' | 'created_at' | 'updated_at'>) => Promise<Modpack | null>;
  updateModpack: (id: string, modpack: Partial<Modpack>) => Promise<Modpack | null>;
  deleteModpack: (id: string) => Promise<boolean>;
  
  selectModpack: (modpack: Modpack | null) => void;
  selectMod: (mod: ModInfo | null) => void;
  
  setFilters: (filters: Partial<ModFilters>) => void;
  clearFilters: () => void;
  
  checkCompatibility: (modpack: Modpack) => Promise<ModpackCompatibility | null>;
  
  // Server Mod Management
  getServerMods: (serverId: string) => Promise<ModInfo[]>;
  addModToServer: (serverId: string, modId: string, version: string) => Promise<boolean>;
  removeModFromServer: (serverId: string, modId: string) => Promise<boolean>;
  applyModpackToServer: (serverId: string, modpackId: string) => Promise<boolean>;
}

const defaultFilters: ModFilters = {
  minecraft_version: '1.21.1',
  loader: 'forge',
  mod_loader: 'forge',
  category: 'all',
  side: 'all',
  search_query: '',
  search: '',
  source: 'all',
  tags: [],
  sort_by: 'popularity',
  sort_order: 'desc',
  min_downloads: 0,
  max_file_size: 0,
  has_client_version: false,
  has_server_version: false,
};

export const useModpackStore = create<ModpackState>((set, get) => ({
  // Initial state
  modpacks: [],
  mods: [],
  minecraftVersions: [],
  searchResults: null,
  loading: false,
  error: null,
  selectedModpack: null,
  selectedMod: null,
  filters: defaultFilters,

  // Load modpacks
  loadModpacks: async () => {
    set({ loading: true, error: null });
    try {
      const response = await modpackApi.getModpacks();
      set({ modpacks: response.success ? (response.data || []) : [], loading: false });
    } catch (error) {
      console.error('Error loading modpacks:', error);
      set({ 
        error: error instanceof Error ? error.message : 'Failed to load modpacks',
        loading: false,
        modpacks: []
      });
    }
  },

  // Load mods with filters
  loadMods: async (filters?: Partial<ModFilters>) => {
    set({ loading: true, error: null });
    try {
      const currentFilters = { ...get().filters, ...filters };
      const response = await modpackApi.searchMods(currentFilters.search_query || '', currentFilters);
      
      // Handle case where response might be undefined or missing mods property
      const mods = response?.success ? (response.data || []) : [];
      const total = response?.total || 0;
      const page = response?.page || 1;
      const per_page = response?.per_page || 20;
      
      set({ 
        mods,
        searchResults: {
          mods,
          total,
          page,
          per_page,
          has_more: page * per_page < total
        },
        filters: currentFilters,
        loading: false 
      });
    } catch (error) {
      console.error('Error loading mods:', error);
      set({ 
        error: error instanceof Error ? error.message : 'Failed to load mods',
        loading: false,
        mods: [],
        searchResults: null
      });
    }
  },

  // Load Minecraft versions
  loadMinecraftVersions: async () => {
    set({ loading: true, error: null });
    try {
      // Use local comprehensive version list instead of API
      const versions = getVersionsForModpack('forge');
      set({ minecraftVersions: versions, loading: false });
    } catch (error) {
      console.error('Error loading Minecraft versions:', error);
      set({ 
        error: error instanceof Error ? error.message : 'Failed to load Minecraft versions',
        loading: false,
        minecraftVersions: getVersionsForModpack('forge') // Fallback to local versions
      });
    }
  },

  // Search mods
  searchMods: async (filters: Partial<ModFilters>) => {
    set({ loading: true, error: null });
    try {
      const currentFilters = { ...get().filters, ...filters };
      const response = await modpackApi.searchMods(currentFilters.search_query || '', currentFilters);
      const mods = response?.success ? (response.data || []) : [];
      const total = response?.total || 0;
      const page = response?.page || 1;
      const per_page = response?.per_page || 20;
      const has_more = response?.has_more || false;
      
      set({ 
        mods,
        searchResults: {
          mods,
          total,
          page,
          per_page,
          has_more
        },
        filters: currentFilters,
        loading: false 
      });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to search mods',
        loading: false 
      });
    }
  },

  // Create modpack
  createModpack: async (modpack) => {
    set({ loading: true, error: null });
    try {
      const response = await modpackApi.createModpack(modpack);
      const newModpack = response?.success ? (response.data || null) : null;
      set((state) => ({
        modpacks: newModpack ? [...state.modpacks, newModpack] : state.modpacks,
        loading: false
      }));
      return newModpack;
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to create modpack',
        loading: false 
      });
      return null;
    }
  },

  // Update modpack
  updateModpack: async (id, modpack) => {
    set({ loading: true, error: null });
    try {
      const response = await modpackApi.updateModpack(id, modpack);
      const updatedModpack = response?.success ? (response.data || null) : null;
      set((state) => ({
        modpacks: state.modpacks.map(mp => mp.id === id ? (updatedModpack || mp) : mp),
        loading: false
      }));
      return updatedModpack;
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to update modpack',
        loading: false 
      });
      return null;
    }
  },

  // Delete modpack
  deleteModpack: async (id) => {
    set({ loading: true, error: null });
    try {
      await modpackApi.deleteModpack(id);
      set((state) => ({
        modpacks: state.modpacks.filter(mp => mp.id !== id),
        loading: false
      }));
      return true;
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to delete modpack',
        loading: false 
      });
      return false;
    }
  },

  // Select modpack
  selectModpack: (modpack) => {
    set({ selectedModpack: modpack });
  },

  // Select mod
  selectMod: (mod) => {
    set({ selectedMod: mod });
  },

  // Set filters
  setFilters: (filters) => {
    set((state) => ({
      filters: { ...state.filters, ...filters }
    }));
  },

  // Clear filters
  clearFilters: () => {
    set({ filters: defaultFilters });
  },

  // Check compatibility
  checkCompatibility: async (modpack) => {
    set({ loading: true, error: null });
    try {
      const modIds = modpack.mods.map(mod => mod.id);
      const response = await modpackApi.checkCompatibility(modIds);
      set({ loading: false });
      return response.success ? response.data : null;
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to check compatibility',
        loading: false 
      });
      return null;
    }
  },

  // Server mod management
  getServerMods: async (serverId) => {
    set({ loading: true, error: null });
    try {
      const response = await modpackApi.getServerMods(serverId);
      const mods = response?.success ? (response.data || []) : [];
      set({ loading: false });
      return mods;
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to load server mods',
        loading: false 
      });
      return [];
    }
  },

  addModToServer: async (serverId, modId, version) => {
    set({ loading: true, error: null });
    try {
      await modpackApi.addModToServer(serverId, modId);
      set({ loading: false });
      return true;
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to add mod to server',
        loading: false 
      });
      return false;
    }
  },

  removeModFromServer: async (serverId, modId) => {
    set({ loading: true, error: null });
    try {
      await modpackApi.removeModFromServer(serverId, modId);
      set({ loading: false });
      return true;
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to remove mod from server',
        loading: false 
      });
      return false;
    }
  },

  applyModpackToServer: async (serverId, modpackId) => {
    set({ loading: true, error: null });
    try {
      await modpackApi.applyModpackToServer(serverId, modpackId);
      set({ loading: false });
      return true;
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to apply modpack to server',
        loading: false 
      });
      return false;
    }
  },
}));
