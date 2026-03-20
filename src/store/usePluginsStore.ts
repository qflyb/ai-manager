import { create } from "zustand";
import type { PluginEntry } from "../types/plugins";
import * as api from "../api/plugins";

interface PluginsStore {
  plugins: PluginEntry[];
  loading: boolean;
  error: string | null;

  fetchPlugins: () => Promise<void>;
}

export const usePluginsStore = create<PluginsStore>((set) => ({
  plugins: [],
  loading: false,
  error: null,

  fetchPlugins: async () => {
    set({ loading: true, error: null });
    try {
      const plugins = await api.listPlugins();
      set({ plugins, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
}));
