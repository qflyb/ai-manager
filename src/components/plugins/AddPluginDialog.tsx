import { useState } from "react";
import { X, FolderOpen, Github, Loader2 } from "lucide-react";
import * as api from "../../api/plugins";

interface AddPluginDialogProps {
  onClose: () => void;
  onComplete: () => void;
}

type TabId = "local" | "github";

export default function AddPluginDialog({
  onClose,
  onComplete,
}: AddPluginDialogProps) {
  const [tab, setTab] = useState<TabId>("local");
  const [localPath, setLocalPath] = useState("");
  const [githubInput, setGithubInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleAddLocal = async () => {
    if (!localPath.trim()) return;
    setLoading(true);
    setError(null);
    try {
      await api.addPluginLocal(localPath.trim());
      onComplete();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleAddGithub = async () => {
    const trimmed = githubInput.trim();
    if (!trimmed) return;

    const parts = trimmed.replace(/^https?:\/\/github\.com\//, "").split("/");
    if (parts.length < 2 || !parts[0] || !parts[1]) {
      setError("Please enter a valid format: owner/repo");
      return;
    }

    setLoading(true);
    setError(null);
    try {
      await api.addPluginGithub(parts[0], parts[1].replace(/\.git$/, ""));
      onComplete();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const tabs: { id: TabId; label: string; icon: typeof FolderOpen }[] = [
    { id: "local", label: "Local Path", icon: FolderOpen },
    { id: "github", label: "GitHub", icon: Github },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
      <div className="w-[460px] rounded-xl bg-white p-6 shadow-xl">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-semibold text-gray-900">Add Plugin</h3>
          <button
            onClick={onClose}
            className="rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Tabs */}
        <div className="mt-4 flex gap-1 rounded-lg bg-gray-100 p-1">
          {tabs.map((t) => (
            <button
              key={t.id}
              onClick={() => {
                setTab(t.id);
                setError(null);
              }}
              className={`flex flex-1 items-center justify-center gap-1.5 rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                tab === t.id
                  ? "bg-white text-gray-900 shadow-sm"
                  : "text-gray-500 hover:text-gray-700"
              }`}
            >
              <t.icon className="h-3.5 w-3.5" />
              {t.label}
            </button>
          ))}
        </div>

        {error && (
          <div className="mt-3 rounded-lg bg-red-50 p-3 text-xs text-red-600">
            {error}
          </div>
        )}

        {/* Local Path Tab */}
        {tab === "local" && (
          <div className="mt-4">
            <label className="block text-sm font-medium text-gray-700">
              Plugin Directory Path
            </label>
            <input
              type="text"
              value={localPath}
              onChange={(e) => setLocalPath(e.target.value)}
              placeholder="D:\plugins\my-plugin"
              className="mt-1.5 w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 placeholder-gray-400 focus:border-violet-500 focus:outline-none focus:ring-1 focus:ring-violet-500"
              onKeyDown={(e) => e.key === "Enter" && handleAddLocal()}
            />
            <p className="mt-1.5 text-xs text-gray-400">
              Path to a directory containing .claude-plugin/plugin.json
            </p>
          </div>
        )}

        {/* GitHub Tab */}
        {tab === "github" && (
          <div className="mt-4">
            <label className="block text-sm font-medium text-gray-700">
              GitHub Repository
            </label>
            <input
              type="text"
              value={githubInput}
              onChange={(e) => setGithubInput(e.target.value)}
              placeholder="owner/repo"
              className="mt-1.5 w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 placeholder-gray-400 focus:border-violet-500 focus:outline-none focus:ring-1 focus:ring-violet-500"
              onKeyDown={(e) => e.key === "Enter" && handleAddGithub()}
            />
            <p className="mt-1.5 text-xs text-gray-400">
              Enter owner/repo or a full GitHub URL
            </p>
          </div>
        )}

        <div className="mt-5 flex justify-end gap-2">
          <button
            onClick={onClose}
            className="rounded-lg border border-gray-200 px-4 py-2 text-sm text-gray-700 hover:bg-gray-50"
          >
            Cancel
          </button>
          <button
            onClick={tab === "local" ? handleAddLocal : handleAddGithub}
            disabled={
              loading ||
              (tab === "local" ? !localPath.trim() : !githubInput.trim())
            }
            className="flex items-center gap-1.5 rounded-lg bg-violet-600 px-4 py-2 text-sm font-medium text-white hover:bg-violet-700 disabled:opacity-50"
          >
            {loading && <Loader2 className="h-3.5 w-3.5 animate-spin" />}
            {loading ? "Adding..." : "Add Plugin"}
          </button>
        </div>
      </div>
    </div>
  );
}
