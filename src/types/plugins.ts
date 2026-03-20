export interface PluginMetadata {
  name: string;
  description: string;
  version: string;
  author: string | null;
  license: string | null;
  keywords: string[] | null;
  repository: string | null;
}

export type PluginSource =
  | { type: "Local"; path: string }
  | { type: "GitHub"; owner: string; repo: string };

export interface PluginEntry {
  id: string;
  source: PluginSource;
  local_path: string;
  metadata: PluginMetadata;
  added_at: string;
}

export interface PluginContents {
  skills: PluginSkillInfo[];
  commands: PluginCommandInfo[];
}

export interface PluginSkillInfo {
  dir_name: string;
  name: string;
  description: string;
  skill_path: string;
  installed_in: string[];
}

export interface PluginCommandInfo {
  file_name: string;
  command_name: string;
  file_path: string;
  installed_in: string[];
}
