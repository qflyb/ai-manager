import { useState } from "react";
import {
  Trash2,
  HardDrive,
  ChevronRight,
  ChevronDown,
  FolderOpen,
  FileText,
} from "lucide-react";
import { useCacheInfo } from "../hooks/useCacheInfo";
import * as api from "../api/cache";
import LoadingSpinner from "../components/ui/LoadingSpinner";
import Badge from "../components/ui/Badge";
import ConfirmDialog from "../components/ui/ConfirmDialog";

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const k = 1024;
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), units.length - 1);
  const value = bytes / Math.pow(k, i);
  return `${value < 10 ? value.toFixed(2) : value < 100 ? value.toFixed(1) : Math.round(value)} ${units[i]}`;
}

interface PendingClear {
  type: "tool" | "all";
  toolId?: string;
  toolName?: string;
  size: number;
}

export default function CacheManagement() {
  const { cacheInfos, loading, error, refetch } = useCacheInfo();
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [clearing, setClearing] = useState<string | null>(null);
  const [clearError, setClearError] = useState<string | null>(null);
  const [pendingClear, setPendingClear] = useState<PendingClear | null>(null);

  const totalSize = cacheInfos.reduce((sum, t) => sum + t.cache_size_bytes, 0);
  const hasAnyCacheData = cacheInfos.some((t) => t.has_cache);

  const toggleExpand = (toolId: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(toolId)) next.delete(toolId);
      else next.add(toolId);
      return next;
    });
  };

  const handleClearTool = (toolId: string, toolName: string, size: number) => {
    setPendingClear({ type: "tool", toolId, toolName, size });
  };

  const handleClearAll = () => {
    setPendingClear({ type: "all", size: totalSize });
  };

  const executeClear = async () => {
    if (!pendingClear) return;
    setPendingClear(null);
    setClearError(null);

    const clearingId = pendingClear.type === "all" ? "__all__" : pendingClear.toolId!;
    setClearing(clearingId);
    try {
      const result =
        pendingClear.type === "all"
          ? await api.clearAllCaches()
          : await api.clearToolCache(pendingClear.toolId!);
      if (result.errors.length > 0) {
        setClearError(
          `Cleared ${formatBytes(result.freed_bytes)}, but some errors occurred:\n${result.errors.join("\n")}`
        );
      }
      refetch();
    } catch (err) {
      setClearError(String(err));
    } finally {
      setClearing(null);
    }
  };

  const confirmMessage =
    pendingClear?.type === "all"
      ? `Clear conversation caches for all AI tools?\nThis will free approximately ${formatBytes(pendingClear.size)}.`
      : `Clear all conversation cache for ${pendingClear?.toolName}?\nThis will free approximately ${formatBytes(pendingClear?.size ?? 0)}.`;

  return (
    <div className="p-8">
      {pendingClear && (
        <ConfirmDialog
          title="Clear Cache"
          message={confirmMessage}
          confirmLabel="Clear"
          onConfirm={executeClear}
          onCancel={() => setPendingClear(null)}
        />
      )}

      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Conversation Cache</h1>
          <p className="mt-1 text-sm text-gray-500">
            Manage conversation history data for AI tools
          </p>
        </div>
        <button
          onClick={handleClearAll}
          disabled={!hasAnyCacheData || clearing !== null}
          className="flex items-center gap-2 rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
        >
          <Trash2 className="h-4 w-4" />
          {clearing === "__all__" ? "Clearing..." : "Clear All"}
        </button>
      </div>

      {loading && <LoadingSpinner text="Scanning conversation caches..." />}

      {(error || clearError) && (
        <div className="mt-6 rounded-lg bg-red-50 p-4 text-sm text-red-600 whitespace-pre-line">
          {error ?? clearError}
        </div>
      )}

      {!loading && cacheInfos.length > 0 && (
        <>
          {/* Summary card */}
          <div className="mt-6 flex items-center gap-4 rounded-xl border border-gray-200 bg-white p-5">
            <div className="rounded-lg bg-indigo-50 p-2.5">
              <HardDrive className="h-5 w-5 text-indigo-600" />
            </div>
            <div>
              <p className="text-sm text-gray-500">Total cache size</p>
              <p className="text-xl font-semibold text-gray-900">
                {formatBytes(totalSize)}
              </p>
            </div>
            <div className="ml-auto text-sm text-gray-400">
              {cacheInfos.filter((t) => t.has_cache).length} / {cacheInfos.length} tools with cache data
            </div>
          </div>

          {/* Tool cache cards */}
          <div className="mt-6 space-y-2">
            {cacheInfos.map((tool) => {
              const isExpanded = expanded.has(tool.tool_id);
              const isClearing =
                clearing === tool.tool_id || clearing === "__all__";

              return (
                <div
                  key={tool.tool_id}
                  className={`overflow-hidden rounded-lg border border-gray-200 bg-white ${
                    !tool.has_cache ? "opacity-60" : ""
                  }`}
                >
                  {/* Tool header */}
                  <button
                    onClick={() => toggleExpand(tool.tool_id)}
                    className="flex w-full items-center gap-3 p-4 text-left transition-colors hover:bg-gray-50"
                  >
                    <div className="text-gray-400">
                      {isExpanded ? (
                        <ChevronDown className="h-4 w-4" />
                      ) : (
                        <ChevronRight className="h-4 w-4" />
                      )}
                    </div>

                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-gray-900">
                          {tool.tool_name}
                        </span>
                        <Badge variant={tool.has_cache ? "info" : "muted"}>
                          {formatBytes(tool.cache_size_bytes)}
                        </Badge>
                      </div>
                      <p className="mt-0.5 text-xs text-gray-500">
                        {tool.cache_paths.length} cache location
                        {tool.cache_paths.length > 1 ? "s" : ""}
                      </p>
                    </div>

                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleClearTool(
                          tool.tool_id,
                          tool.tool_name,
                          tool.cache_size_bytes
                        );
                      }}
                      disabled={!tool.has_cache || isClearing}
                      className="flex items-center gap-1 rounded-md px-3 py-1.5 text-xs font-medium text-red-600 transition-colors hover:bg-red-50 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                      {isClearing ? "Clearing..." : "Clear"}
                    </button>
                  </button>

                  {/* Expanded: cache path details */}
                  {isExpanded && (
                    <div className="border-t border-gray-100 bg-gray-50/50">
                      {tool.cache_paths.map((cachePath) => (
                        <div
                          key={cachePath.full_path}
                          className="flex items-center gap-3 border-b border-gray-100 px-4 py-3 last:border-b-0"
                        >
                          <div className="ml-7 text-gray-400">
                            {cachePath.full_path.includes(".") &&
                            !cachePath.full_path.endsWith("/") ? (
                              <FileText className="h-3.5 w-3.5" />
                            ) : (
                              <FolderOpen className="h-3.5 w-3.5" />
                            )}
                          </div>
                          <div className="min-w-0 flex-1">
                            <span className="text-sm text-gray-700">
                              {cachePath.label}
                            </span>
                            <p
                              className="mt-0.5 truncate text-xs text-gray-400"
                              title={cachePath.full_path}
                            >
                              {cachePath.full_path}
                            </p>
                          </div>
                          <div className="text-xs text-gray-500">
                            {cachePath.exists ? (
                              formatBytes(cachePath.size_bytes)
                            ) : (
                              <span className="text-gray-300">N/A</span>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
