import type { PluginEntry, PluginContents } from "../types/plugins";
import { tauriInvoke } from "./invoke";

export async function addPluginLocal(path: string): Promise<PluginEntry> {
  return tauriInvoke<PluginEntry>("add_plugin_local", { path });
}

export async function addPluginGithub(
  owner: string,
  repo: string
): Promise<PluginEntry> {
  return tauriInvoke<PluginEntry>("add_plugin_github", { owner, repo });
}

export async function listPlugins(): Promise<PluginEntry[]> {
  return tauriInvoke<PluginEntry[]>("list_plugins");
}

export async function removePlugin(pluginId: string): Promise<void> {
  return tauriInvoke("remove_plugin", { pluginId });
}

export async function updatePlugin(pluginId: string): Promise<PluginEntry> {
  return tauriInvoke<PluginEntry>("update_plugin", { pluginId });
}

export async function listPluginContents(
  pluginId: string
): Promise<PluginContents> {
  return tauriInvoke<PluginContents>("list_plugin_contents", { pluginId });
}

export async function installPluginSkill(
  pluginId: string,
  skillDirName: string,
  toolId: string
): Promise<void> {
  return tauriInvoke("install_plugin_skill", { pluginId, skillDirName, toolId });
}

export async function installPluginSkillToAll(
  pluginId: string,
  skillDirName: string
): Promise<void> {
  return tauriInvoke("install_plugin_skill_to_all", { pluginId, skillDirName });
}

export async function installPluginCommand(
  pluginId: string,
  commandFile: string,
  toolId: string
): Promise<void> {
  return tauriInvoke("install_plugin_command", {
    pluginId,
    commandFile,
    toolId,
  });
}

export async function installPluginCommandToAll(
  pluginId: string,
  commandFile: string
): Promise<void> {
  return tauriInvoke("install_plugin_command_to_all", {
    pluginId,
    commandFile,
  });
}

export async function removePluginSkill(
  pluginId: string,
  skillDirName: string,
  toolId: string
): Promise<void> {
  return tauriInvoke("remove_plugin_skill", {
    pluginId,
    skillDirName,
    toolId,
  });
}

export async function removePluginCommand(
  pluginId: string,
  commandFile: string,
  toolId: string
): Promise<void> {
  return tauriInvoke("remove_plugin_command", {
    pluginId,
    commandFile,
    toolId,
  });
}
