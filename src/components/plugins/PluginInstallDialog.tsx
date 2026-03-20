import { useState } from "react";
import { X, Download, DownloadCloud } from "lucide-react";
import type { AiTool } from "../../types/skills";

interface PluginInstallDialogProps {
  title: string;
  tools: AiTool[];
  installedIn: string[];
  onInstall: (toolId: string) => Promise<void>;
  onInstallAll: () => Promise<void>;
  onClose: () => void;
  onComplete: () => void;
  /** Filter tools - "skills" for skill items, "commands" for command items */
  filterType: "skills" | "commands";
}

export default function PluginInstallDialog({
  title,
  tools,
  installedIn,
  onInstall,
  onInstallAll,
  onClose,
  onComplete,
  filterType,
}: PluginInstallDialogProps) {
  const [installing, setInstalling] = useState<string | null>(null);
  const [installingAll, setInstallingAll] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const filteredTools = tools.filter((t) => {
    if (filterType === "skills") {
      return t.capability === "skills" && t.detected;
    }
    // For commands, only show tools that have commands support
    // Currently only Claude Code, but filter by detected
    return t.detected;
  });

  const handleInstall = async (toolId: string) => {
    setInstalling(toolId);
    setError(null);
    try {
      await onInstall(toolId);
      onComplete();
    } catch (e) {
      setError(String(e));
    } finally {
      setInstalling(null);
    }
  };

  const handleInstallAll = async () => {
    setInstallingAll(true);
    setError(null);
    try {
      await onInstallAll();
      onComplete();
    } catch (e) {
      setError(String(e));
    } finally {
      setInstallingAll(false);
    }
  };

  const allInstalled = filteredTools.every((t) => installedIn.includes(t.id));

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
      <div className="w-[420px] rounded-xl bg-white p-6 shadow-xl">
        <div className="flex items-center justify-between">
          <h3 className="text-base font-semibold text-gray-900">{title}</h3>
          <button
            onClick={onClose}
            className="rounded-md p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <p className="mt-2 text-sm text-gray-500">
          Select which AI tools to install to:
        </p>

        {error && (
          <div className="mt-3 rounded-lg bg-red-50 p-3 text-xs text-red-600">
            {error}
          </div>
        )}

        <div className="mt-4 space-y-2">
          {filteredTools.map((tool) => {
            const isInstalled = installedIn.includes(tool.id);
            return (
              <div
                key={tool.id}
                className="flex items-center justify-between rounded-lg border border-gray-200 px-4 py-3"
              >
                <span className="text-sm font-medium text-gray-700">
                  {tool.name}
                </span>
                {isInstalled ? (
                  <span className="text-xs text-emerald-600">Installed</span>
                ) : (
                  <button
                    onClick={() => handleInstall(tool.id)}
                    disabled={installing !== null || installingAll}
                    className="flex items-center gap-1 rounded-md bg-violet-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-violet-700 disabled:opacity-50"
                  >
                    <Download className="h-3 w-3" />
                    {installing === tool.id ? "Installing..." : "Install"}
                  </button>
                )}
              </div>
            );
          })}
        </div>

        <div className="mt-5 flex items-center justify-between">
          {!allInstalled && (
            <button
              onClick={handleInstallAll}
              disabled={installing !== null || installingAll}
              className="flex items-center gap-1.5 rounded-lg bg-violet-50 px-3 py-2 text-xs font-medium text-violet-700 hover:bg-violet-100 disabled:opacity-50"
            >
              <DownloadCloud className="h-3.5 w-3.5" />
              {installingAll ? "Installing..." : "Install to All"}
            </button>
          )}
          <div className="ml-auto">
            <button
              onClick={onClose}
              className="rounded-lg border border-gray-200 px-4 py-2 text-sm text-gray-700 hover:bg-gray-50"
            >
              Done
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
