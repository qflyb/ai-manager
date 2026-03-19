export type ToolCapability = "skills" | "config-only" | "detected-only";

export interface AiTool {
  id: string;
  name: string;
  config_dir: string;
  capability: ToolCapability;
  skills_dir: string | null;
  config_files: ConfigFile[];
  skill_count: number;
  detected: boolean;
}

export interface ConfigFile {
  name: string;
  path: string;
  format: "json" | "toml" | "markdown" | "yaml" | "unknown";
}

export interface Skill {
  name: string;
  description: string;
  allowed_tools: string | null;
  dir_name: string;
  dir_path: string;
  skill_file_path: string;
  is_symlink: boolean;
  symlink_target: string | null;
  has_references: boolean;
  has_agents: boolean;
  has_scripts: boolean;
  installed_in: string[];
  disabled: boolean;
}

export interface SkillContent {
  frontmatter: Record<string, string>;
  markdown_body: string;
  raw_content: string;
  references: ReferenceFile[];
}

export interface ReferenceFile {
  name: string;
  path: string;
  content: string;
}

export interface EditorInfo {
  id: string;
  label: string;
}
