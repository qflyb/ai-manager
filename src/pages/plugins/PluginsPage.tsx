import { useState } from "react";
import { Plus, RefreshCw, Trash2 } from "lucide-react";
import { usePlugins } from "../../hooks/usePlugins";
import PluginCard from "../../components/plugins/PluginCard";
import AddPluginDialog from "../../components/plugins/AddPluginDialog";
import SearchInput from "../../components/ui/SearchInput";
import LoadingSpinner from "../../components/ui/LoadingSpinner";
import EmptyState from "../../components/ui/EmptyState";
import Badge from "../../components/ui/Badge";
import * as api from "../../api/plugins";

export default function PluginsPage() {
  const { plugins, loading, error, refetch } = usePlugins();
  const [search, setSearch] = useState("");
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const filtered = plugins.filter(
    (p) =>
      p.metadata.name.toLowerCase().includes(search.toLowerCase()) ||
      p.metadata.description.toLowerCase().includes(search.toLowerCase()) ||
      p.id.toLowerCase().includes(search.toLowerCase())
  );

  const handleRemove = async (pluginId: string, pluginName: string) => {
    if (!confirm(`Remove plugin "${pluginName}"? This will not uninstall skills or commands already added to AI tools.`)) {
      return;
    }
    setActionLoading(pluginId);
    setActionError(null);
    try {
      await api.removePlugin(pluginId);
      await refetch();
    } catch (e) {
      setActionError(String(e));
    } finally {
      setActionLoading(null);
    }
  };

  const handleUpdate = async (pluginId: string) => {
    setActionLoading(pluginId);
    setActionError(null);
    try {
      await api.updatePlugin(pluginId);
      await refetch();
    } catch (e) {
      setActionError(String(e));
    } finally {
      setActionLoading(null);
    }
  };

  return (
    <div className="p-8">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Plugins</h1>
          <p className="mt-1 text-sm text-gray-500">
            Manage Claude Code compatible plugins from local paths or GitHub
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Badge variant="info">{plugins.length} plugins</Badge>
          <div className="w-64">
            <SearchInput
              value={search}
              onChange={setSearch}
              placeholder="Search plugins..."
            />
          </div>
          <button
            onClick={() => setShowAddDialog(true)}
            className="flex items-center gap-1.5 rounded-lg bg-violet-600 px-4 py-2 text-sm font-medium text-white hover:bg-violet-700"
          >
            <Plus className="h-4 w-4" />
            Add Plugin
          </button>
        </div>
      </div>

      {actionError && (
        <div className="mt-4 rounded-lg bg-red-50 p-4 text-sm text-red-600">
          {actionError}
        </div>
      )}

      {loading && <LoadingSpinner text="Loading plugins..." />}
      {error && (
        <div className="mt-6 rounded-lg bg-red-50 p-4 text-sm text-red-600">
          {error}
        </div>
      )}

      {!loading && filtered.length === 0 && (
        <div className="mt-8">
          <EmptyState
            title="No plugins added"
            description="Add plugins from a local directory or GitHub repository"
          />
        </div>
      )}

      <div className="mt-6 space-y-3">
        {filtered.map((plugin) => (
          <div key={plugin.id} className="flex items-start gap-2">
            <div className="flex-1">
              <PluginCard plugin={plugin} />
            </div>
            <div className="flex flex-shrink-0 gap-1.5 pt-3">
              {plugin.source.type === "GitHub" && (
                <button
                  onClick={() => handleUpdate(plugin.id)}
                  disabled={actionLoading === plugin.id}
                  title="Update from GitHub"
                  className="rounded-md border border-gray-200 p-1.5 text-gray-400 hover:bg-gray-50 hover:text-gray-600 disabled:opacity-50"
                >
                  <RefreshCw
                    className={`h-3.5 w-3.5 ${actionLoading === plugin.id ? "animate-spin" : ""}`}
                  />
                </button>
              )}
              <button
                onClick={() => handleRemove(plugin.id, plugin.metadata.name)}
                disabled={actionLoading === plugin.id}
                title="Remove plugin"
                className="rounded-md border border-gray-200 p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-500 disabled:opacity-50"
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </div>
          </div>
        ))}
      </div>

      {showAddDialog && (
        <AddPluginDialog
          onClose={() => setShowAddDialog(false)}
          onComplete={() => {
            setShowAddDialog(false);
            refetch();
          }}
        />
      )}
    </div>
  );
}
