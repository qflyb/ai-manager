import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  FileText,
  Terminal,
  Download,
  DownloadCloud,
  Trash2,
  Github,
  FolderOpen,
} from "lucide-react";
import { usePlugins } from "../../hooks/usePlugins";
import { usePluginContents } from "../../hooks/usePluginContents";
import { useAiTools } from "../../hooks/useAiTools";
import PluginInstallDialog from "../../components/plugins/PluginInstallDialog";
import LoadingSpinner from "../../components/ui/LoadingSpinner";
import Badge from "../../components/ui/Badge";
import type { PluginSkillInfo, PluginCommandInfo } from "../../types/plugins";
import * as api from "../../api/plugins";

type InstallTarget =
  | { kind: "skill"; item: PluginSkillInfo }
  | { kind: "command"; item: PluginCommandInfo };

export default function PluginDetailPage() {
  const { pluginId } = useParams<{ pluginId: string }>();
  const navigate = useNavigate();
  const { plugins } = usePlugins();
  const { contents, loading, error, refetch } = usePluginContents(pluginId);
  const { tools } = useAiTools();
  const [installTarget, setInstallTarget] = useState<InstallTarget | null>(
    null
  );
  const [batchLoading, setBatchLoading] = useState<string | null>(null);
  const [batchError, setBatchError] = useState<string | null>(null);

  const plugin = plugins.find((p) => p.id === pluginId);

  if (!plugin) {
    return (
      <div className="p-8">
        <button
          onClick={() => navigate("/plugins")}
          className="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to Plugins
        </button>
        <div className="mt-6 text-sm text-gray-500">Plugin not found.</div>
      </div>
    );
  }

  const isGithub = plugin.source.type === "GitHub";

  const handleBatchInstallSkills = async () => {
    if (!pluginId || !contents) return;
    setBatchLoading("skills");
    setBatchError(null);
    try {
      for (const skill of contents.skills) {
        await api.installPluginSkillToAll(pluginId, skill.dir_name);
      }
      await refetch();
    } catch (e) {
      setBatchError(String(e));
    } finally {
      setBatchLoading(null);
    }
  };

  const handleBatchInstallCommands = async () => {
    if (!pluginId || !contents) return;
    setBatchLoading("commands");
    setBatchError(null);
    try {
      for (const cmd of contents.commands) {
        await api.installPluginCommandToAll(pluginId, cmd.file_name);
      }
      await refetch();
    } catch (e) {
      setBatchError(String(e));
    } finally {
      setBatchLoading(null);
    }
  };

  const handleRemoveSkill = async (
    skillDirName: string,
    toolId: string
  ) => {
    if (!pluginId) return;
    try {
      await api.removePluginSkill(pluginId, skillDirName, toolId);
      await refetch();
    } catch (e) {
      setBatchError(String(e));
    }
  };

  const handleRemoveCommand = async (
    commandFile: string,
    toolId: string
  ) => {
    if (!pluginId) return;
    try {
      await api.removePluginCommand(pluginId, commandFile, toolId);
      await refetch();
    } catch (e) {
      setBatchError(String(e));
    }
  };

  const skillsTools = tools.filter(
    (t) => t.capability === "skills" && t.detected
  );

  return (
    <div className="p-8">
      <button
        onClick={() => navigate("/plugins")}
        className="flex items-center gap-1 text-sm text-gray-500 hover:text-gray-700"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to Plugins
      </button>

      {/* Plugin Header */}
      <div className="mt-6 rounded-xl border border-gray-200 bg-white p-6">
        <div className="flex items-start gap-4">
          <div className="rounded-lg bg-violet-50 p-3">
            {isGithub ? (
              <Github className="h-6 w-6 text-violet-500" />
            ) : (
              <FolderOpen className="h-6 w-6 text-violet-500" />
            )}
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-3">
              <h1 className="text-xl font-bold text-gray-900">
                {plugin.metadata.name}
              </h1>
              {plugin.metadata.version && (
                <Badge variant="muted">v{plugin.metadata.version}</Badge>
              )}
            </div>
            {plugin.metadata.description && (
              <p className="mt-1 text-sm text-gray-500">
                {plugin.metadata.description}
              </p>
            )}
            <div className="mt-3 flex flex-wrap gap-2 text-xs text-gray-400">
              {plugin.metadata.author && (
                <span>Author: {plugin.metadata.author}</span>
              )}
              {plugin.metadata.license && (
                <span>License: {plugin.metadata.license}</span>
              )}
              {isGithub && plugin.source.type === "GitHub" && (
                <span>
                  Source: {plugin.source.owner}/{plugin.source.repo}
                </span>
              )}
              {!isGithub && plugin.source.type === "Local" && (
                <span>Path: {plugin.source.path}</span>
              )}
            </div>
            {plugin.metadata.keywords && plugin.metadata.keywords.length > 0 && (
              <div className="mt-2 flex flex-wrap gap-1">
                {plugin.metadata.keywords.map((kw) => (
                  <Badge key={kw} variant="muted">
                    {kw}
                  </Badge>
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {loading && <LoadingSpinner text="Loading plugin contents..." />}
      {error && (
        <div className="mt-4 rounded-lg bg-red-50 p-4 text-sm text-red-600">
          {error}
        </div>
      )}
      {batchError && (
        <div className="mt-4 rounded-lg bg-red-50 p-4 text-sm text-red-600">
          {batchError}
        </div>
      )}

      {contents && (
        <>
          {/* Skills Section */}
          {contents.skills.length > 0 && (
            <div className="mt-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <FileText className="h-5 w-5 text-indigo-500" />
                  <h2 className="text-lg font-semibold text-gray-900">
                    Skills
                  </h2>
                  <Badge variant="info">{contents.skills.length}</Badge>
                </div>
                <button
                  onClick={handleBatchInstallSkills}
                  disabled={batchLoading === "skills"}
                  className="flex items-center gap-1.5 rounded-lg bg-violet-50 px-3 py-2 text-xs font-medium text-violet-700 hover:bg-violet-100 disabled:opacity-50"
                >
                  <DownloadCloud className="h-3.5 w-3.5" />
                  {batchLoading === "skills"
                    ? "Installing..."
                    : "Install All to All Tools"}
                </button>
              </div>

              <div className="mt-3 space-y-2">
                {contents.skills.map((skill) => (
                  <div
                    key={skill.dir_name}
                    className="flex items-center justify-between rounded-lg border border-gray-200 bg-white px-4 py-3"
                  >
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-gray-900">
                          {skill.name}
                        </span>
                        {skill.installed_in.length > 0 && (
                          <Badge variant="success">
                            {skill.installed_in.length} tool
                            {skill.installed_in.length > 1 ? "s" : ""}
                          </Badge>
                        )}
                      </div>
                      {skill.description && (
                        <p className="mt-0.5 text-xs text-gray-500">
                          {skill.description}
                        </p>
                      )}
                      {skill.installed_in.length > 0 && (
                        <div className="mt-1.5 flex flex-wrap gap-1">
                          {skill.installed_in.map((toolId) => {
                            const tool = tools.find((t) => t.id === toolId);
                            return (
                              <span
                                key={toolId}
                                className="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2 py-0.5 text-xs text-emerald-700"
                              >
                                {tool?.name || toolId}
                                <button
                                  onClick={() =>
                                    handleRemoveSkill(skill.dir_name, toolId)
                                  }
                                  className="ml-0.5 text-emerald-400 hover:text-red-500"
                                  title="Remove from this tool"
                                >
                                  <Trash2 className="h-2.5 w-2.5" />
                                </button>
                              </span>
                            );
                          })}
                        </div>
                      )}
                    </div>
                    <button
                      onClick={() =>
                        setInstallTarget({ kind: "skill", item: skill })
                      }
                      className="flex flex-shrink-0 items-center gap-1 rounded-md bg-violet-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-violet-700"
                    >
                      <Download className="h-3 w-3" />
                      Install
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Commands Section */}
          {contents.commands.length > 0 && (
            <div className="mt-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Terminal className="h-5 w-5 text-amber-500" />
                  <h2 className="text-lg font-semibold text-gray-900">
                    Commands
                  </h2>
                  <Badge variant="info">{contents.commands.length}</Badge>
                </div>
                <button
                  onClick={handleBatchInstallCommands}
                  disabled={batchLoading === "commands"}
                  className="flex items-center gap-1.5 rounded-lg bg-violet-50 px-3 py-2 text-xs font-medium text-violet-700 hover:bg-violet-100 disabled:opacity-50"
                >
                  <DownloadCloud className="h-3.5 w-3.5" />
                  {batchLoading === "commands"
                    ? "Installing..."
                    : "Install All to All Tools"}
                </button>
              </div>

              <div className="mt-3 space-y-2">
                {contents.commands.map((cmd) => (
                  <div
                    key={cmd.file_name}
                    className="flex items-center justify-between rounded-lg border border-gray-200 bg-white px-4 py-3"
                  >
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-gray-900">
                          /{cmd.command_name}
                        </span>
                        {cmd.installed_in.length > 0 && (
                          <Badge variant="success">
                            {cmd.installed_in.length} tool
                            {cmd.installed_in.length > 1 ? "s" : ""}
                          </Badge>
                        )}
                      </div>
                      <p className="mt-0.5 text-xs text-gray-400">
                        {cmd.file_name}
                      </p>
                      {cmd.installed_in.length > 0 && (
                        <div className="mt-1.5 flex flex-wrap gap-1">
                          {cmd.installed_in.map((toolId) => {
                            const tool = tools.find((t) => t.id === toolId);
                            return (
                              <span
                                key={toolId}
                                className="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2 py-0.5 text-xs text-emerald-700"
                              >
                                {tool?.name || toolId}
                                <button
                                  onClick={() =>
                                    handleRemoveCommand(cmd.file_name, toolId)
                                  }
                                  className="ml-0.5 text-emerald-400 hover:text-red-500"
                                  title="Remove from this tool"
                                >
                                  <Trash2 className="h-2.5 w-2.5" />
                                </button>
                              </span>
                            );
                          })}
                        </div>
                      )}
                    </div>
                    <button
                      onClick={() =>
                        setInstallTarget({ kind: "command", item: cmd })
                      }
                      className="flex flex-shrink-0 items-center gap-1 rounded-md bg-violet-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-violet-700"
                    >
                      <Download className="h-3 w-3" />
                      Install
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}

          {contents.skills.length === 0 && contents.commands.length === 0 && (
            <div className="mt-6 rounded-lg border border-gray-200 bg-gray-50 p-8 text-center text-sm text-gray-500">
              This plugin does not contain any skills or commands.
            </div>
          )}
        </>
      )}

      {/* Install Dialog */}
      {installTarget && pluginId && (
        <PluginInstallDialog
          title={
            installTarget.kind === "skill"
              ? `Install Skill "${installTarget.item.name}"`
              : `Install Command "/${(installTarget.item as PluginCommandInfo).command_name}"`
          }
          tools={installTarget.kind === "skill" ? skillsTools : tools}
          installedIn={
            installTarget.kind === "skill"
              ? installTarget.item.installed_in
              : installTarget.item.installed_in
          }
          filterType={installTarget.kind === "skill" ? "skills" : "commands"}
          onInstall={async (toolId) => {
            if (installTarget.kind === "skill") {
              await api.installPluginSkill(
                pluginId,
                (installTarget.item as PluginSkillInfo).dir_name,
                toolId
              );
            } else {
              await api.installPluginCommand(
                pluginId,
                (installTarget.item as PluginCommandInfo).file_name,
                toolId
              );
            }
          }}
          onInstallAll={async () => {
            if (installTarget.kind === "skill") {
              await api.installPluginSkillToAll(
                pluginId,
                (installTarget.item as PluginSkillInfo).dir_name
              );
            } else {
              await api.installPluginCommandToAll(
                pluginId,
                (installTarget.item as PluginCommandInfo).file_name
              );
            }
          }}
          onClose={() => setInstallTarget(null)}
          onComplete={() => {
            setInstallTarget(null);
            refetch();
          }}
        />
      )}
    </div>
  );
}
