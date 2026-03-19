import type { AiTool, Skill, SkillContent, EditorInfo } from "../types/skills";

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!(window as any).__TAURI_INTERNALS__) {
    throw new Error(
      `Tauri runtime not available. Please run with "bun run tauri dev" instead of "bun run dev".`
    );
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

export async function scanAiTools(): Promise<AiTool[]> {
  return tauriInvoke<AiTool[]>("scan_ai_tools");
}

export async function listSkills(toolId: string): Promise<Skill[]> {
  return tauriInvoke<Skill[]>("list_skills", { toolId });
}

export async function readSkill(skillPath: string): Promise<SkillContent> {
  return tauriInvoke<SkillContent>("read_skill", { skillPath });
}

export async function getHubSkills(): Promise<Skill[]> {
  return tauriInvoke<Skill[]>("get_hub_skills");
}

export async function installSkill(
  hubSkillName: string,
  toolId: string
): Promise<void> {
  return tauriInvoke("install_skill", { hubSkillName, toolId });
}

export async function removeSkill(
  toolId: string,
  skillName: string
): Promise<void> {
  return tauriInvoke("remove_skill", { toolId, skillName });
}

export async function toggleSkill(
  toolId: string,
  skillName: string,
  enabled: boolean
): Promise<void> {
  return tauriInvoke("toggle_skill", { toolId, skillName, enabled });
}

export async function readConfigFile(filePath: string): Promise<string> {
  return tauriInvoke<string>("read_config_file", { filePath });
}

export async function detectEditors(): Promise<EditorInfo[]> {
  return tauriInvoke<EditorInfo[]>("detect_editors");
}

export async function openInEditor(
  filePath: string,
  editor: string
): Promise<void> {
  return tauriInvoke("open_in_editor", { filePath, editor });
}
