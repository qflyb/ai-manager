import { useEffect } from "react";
import { usePluginsStore } from "../store/usePluginsStore";

export function usePlugins() {
  const { plugins, loading, error, fetchPlugins } = usePluginsStore();

  useEffect(() => {
    if (plugins.length === 0 && !loading) {
      fetchPlugins();
    }
  }, []);

  return { plugins, loading, error, refetch: fetchPlugins };
}
