import { useState, useCallback, useEffect } from "react";
import type { PluginContents } from "../types/plugins";
import * as api from "../api/plugins";

export function usePluginContents(pluginId: string | undefined) {
  const [contents, setContents] = useState<PluginContents | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetch = useCallback(async () => {
    if (!pluginId) return;
    setLoading(true);
    setError(null);
    try {
      const data = await api.listPluginContents(pluginId);
      setContents(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [pluginId]);

  useEffect(() => {
    fetch();
  }, [fetch]);

  return { contents, loading, error, refetch: fetch };
}
