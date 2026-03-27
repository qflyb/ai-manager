# AI Skills Manager

## Overview

AI Skills Manager is the core module of the ai-manager desktop application. It provides unified management of Skills across all AI coding tools installed on the current machine, with support for Windows, macOS, and Linux.

## Tech Stack

- **Frontend**: React 19 + TypeScript + Tailwind CSS 4 + Vite 7
- **Backend**: Tauri v2 (Rust)
- **Routing**: react-router-dom (Hash Router)
- **State management**: Zustand
- **Markdown rendering**: react-markdown + remark-gfm + remark-frontmatter
- **Icons**: lucide-react

## Supported AI Tools

### Tools with Skills Support

| Tool | Config Directory | Skills Directory | Commands/Prompts Directory | Config Files |
|------|-----------------|-----------------|----------------------------|--------------|
| Claude Code | `~/.claude/` | `skills/` | `commands/` | `settings.json`, `CLAUDE.md` |
| Codex (OpenAI) | `~/.codex/` | `skills/.system/` | `prompts/` | `config.toml`, `AGENTS.md` |
| Google Gemini | `~/.gemini/` | `skills/` | - | `settings.json`, `GEMINI.md` |
| GitHub Copilot | `~/.copilot/` | `skills/` | - | - |

### Config-Only Tools

| Tool | Config Directory | Main Config |
|------|-----------------|-------------|
| Cursor | Platform-specific (see cross-platform section) | IDE config |
| CodeBuddy | `~/.codebuddy/` | `mcp.json`, `argv.json` |
| CodeGeex | `~/.codegeex/` | agent config |
| MarsCode | `~/.marscode/` | `config.json` |
| Lingma | `~/.lingma/` | `lingma_mcp.json` |
| Kiro | `~/.kiro/` | settings |
| Gongfeng Copilot | `~/.gongfeng_copilot/` | `config.json` |
| CodingCopilot | `~/.codingCopilot/` | MCP config |

### Shared Skills Hub

Location: `~/.agents/skills/`

Multiple tools share skills from this central repository via symlinks.

## Skill File Format

```
skills/skill-name/
├── SKILL.md          # YAML frontmatter + Markdown body
├── references/       # Optional reference files
├── agents/           # Optional agent configs (e.g. openai.yaml)
├── scripts/          # Optional scripts
└── LICENSE.txt
```

### SKILL.md Format

```markdown
---
name: skill-name
description: Skill description
allowed-tools: Bash(tool:*), Read, Write
---

# Skill Content
Instructions and documentation in Markdown...
```

## Cross-Platform Support

### Path Resolution

Most tools resolve paths relative to the user's home directory, consistently across all three platforms (`~/.claude/`, `~/.codex/`, etc.). The Rust side uses `dirs::home_dir()` to obtain the home directory and `std::path::PathBuf` for path joining.

### Cursor Platform-Specific Paths

| Platform | Path |
|----------|------|
| Windows | `%APPDATA%\Cursor\` |
| macOS | `~/Library/Application Support/Cursor/` |
| Linux | `~/.config/Cursor/` |

### Symlink Handling

| Platform | API | Permission Requirement |
|----------|-----|-----------------------|
| Linux/macOS | `std::os::unix::fs::symlink()` | None |
| Windows | `std::os::windows::fs::symlink_dir()` | Developer Mode or administrator |

Use conditional compilation `#[cfg(unix)]` / `#[cfg(windows)]` to handle platform differences.

## Project Structure

### Frontend

```
src/
├── main.tsx                         # Hash Router entry point
├── App.tsx                          # AppShell layout (Sidebar + Outlet)
├── types/
│   ├── index.ts                     # Shared types
│   └── skills.ts                    # Skills module types (AiTool, Skill, SkillContent)
├── store/
│   └── useSkillsStore.ts            # Zustand store (tools, hubSkills)
├── api/
│   └── skills.ts                    # Tauri invoke() wrappers (12 APIs)
├── hooks/
│   ├── useAiTools.ts                # Fetch AI tool list
│   ├── useSkills.ts                 # Fetch skills for a given tool
│   ├── useAllSkills.ts              # Fetch all skills grouped by name across tools
│   └── useSkillContent.ts           # Fetch full skill content
├── components/
│   ├── layout/
│   │   └── Sidebar.tsx              # Sidebar (module nav + sub-nav)
│   ├── ui/                          # Generic UI components
│   │   ├── Badge.tsx                # Badge (5 variants)
│   │   ├── StatusDot.tsx            # Status indicator dot
│   │   ├── EmptyState.tsx           # Empty state placeholder
│   │   ├── LoadingSpinner.tsx       # Loading indicator
│   │   ├── MarkdownViewer.tsx       # Markdown renderer
│   │   └── SearchInput.tsx          # Search input
│   └── skills/                      # Skills module components
│       ├── ToolCard.tsx             # Tool card
│       ├── SkillCard.tsx            # Skill card
│       ├── SkillActions.tsx         # Skill actions (disable/remove)
│       ├── InstallDialog.tsx        # Install-to-tool dialog
│       └── ConfigViewer.tsx         # Config file viewer
└── pages/
    ├── Home.tsx                     # Global home (module entry)
    ├── Settings.tsx                 # Global settings
    └── skills/                      # Skills module pages
        ├── SkillsDashboard.tsx      # Tool overview (card grid, grouped by capability)
        ├── ToolDetailPage.tsx       # Tool detail (skills list + config)
        ├── SkillDetailPage.tsx      # Skill detail (Markdown rendering + metadata)
        ├── BySkillPage.tsx          # Group by skill name (cross-tool view + bulk actions)
        └── HubPage.tsx             # Shared Skills Hub (install management)
```

### Rust Backend

```
src-tauri/src/
├── main.rs              # App entry point
├── lib.rs               # Tauri Builder, registers all commands
└── skills/              # Skills module
    ├── mod.rs           # Module exports
    ├── commands.rs      # 12 Tauri command functions
    ├── models.rs        # Data structs (AiToolInfo, SkillInfo, SkillGroup, SkillContent, etc.)
    ├── registry.rs      # Tool registry (12 tools, cross-platform path resolution)
    ├── parser.rs        # SKILL.md YAML frontmatter parser
    └── fs_utils.rs      # Cross-platform symlink operations (create/delete/detect)
```

## Route Structure

```
/                                    → Global home (module entry navigation)
/skills                              → Skills Dashboard (tool overview)
/skills/tools/:toolId                → Tool detail (skills list + config)
/skills/tools/:toolId/:skillName     → Skill detail (Markdown rendering)
/skills/by-skill                     → By Skill (cross-tool grouped view)
/skills/hub                          → Shared Skills Hub
/skills/hub/:skillName               → Hub skill detail
/settings                            → Global settings
```

The app uses a modular routing design to support future modules such as `/mcp/` and `/rules/`.

## Tauri Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `scan_ai_tools` | - | `Vec<AiToolInfo>` | Scan all registered AI tools on the machine |
| `list_skills` | `tool_id` | `Vec<SkillInfo>` | List skills for a given tool |
| `list_all_skills` | - | `Vec<SkillGroup>` | List all skills across tools, grouped by `dir_name` |
| `read_skill` | `skill_path` | `SkillContent` | Read SKILL.md and references |
| `get_hub_skills` | - | `Vec<SkillInfo>` | Read hub skills (including install status) |
| `install_skill` | `hub_skill_name`, `tool_id` | `()` | Install skill from hub (create symlink) |
| `remove_skill` | `tool_id`, `skill_name` | `()` | Remove skill (unlink symlink or delete directory) |
| `remove_skill_from_all` | `skill_name` | `()` | Remove skill from all tools at once (best-effort) |
| `toggle_skill` | `tool_id`, `skill_name`, `enabled` | `()` | Enable/disable (rename with `.disabled-` prefix) |
| `read_config_file` | `file_path` | `String` | Read config file contents |
| `list_commands` | `tool_id` | `Vec<CommandInfo>` | List command or prompt files for a supported tool |
| `read_command_file` | `file_path` | `String` | Read a command or prompt file |
| `detect_editors` | - | `Vec<EditorInfo>` | Detect available code editors in PATH |
| `open_in_editor` | `file_path`, `editor` | `()` | Open file in specified editor |
| `remove_command` | `tool_id`, `command_file` | `()` | Remove a command or prompt file from a supported tool |

## Data Models

### AiTool (frontend type)

```typescript
interface AiTool {
  id: string;              // Tool identifier (e.g. "claude", "codex")
  name: string;            // Display name
  config_dir: string;      // Absolute path to config directory
  capability: "skills" | "config-only" | "detected-only";
  skills_dir: string | null;
  commands_dir: string | null;
  config_files: ConfigFile[];
  skill_count: number;
  detected: boolean;       // Whether the directory exists
}
```

### Skill

```typescript
interface Skill {
  name: string;            // Parsed from YAML frontmatter
  description: string;
  allowed_tools: string | null;
  dir_name: string;        // Directory name (slug)
  dir_path: string;        // Absolute path
  skill_file_path: string; // Absolute path to SKILL.md
  is_symlink: boolean;
  symlink_target: string | null;
  has_references: boolean;
  has_agents: boolean;
  has_scripts: boolean;
  installed_in: string[];  // Tools this skill is installed in (hub skills only)
}
```

### SkillGroup (cross-tool grouped view)

```typescript
interface SkillGroup {
  dir_name: string;          // Grouping key (directory name)
  name: string;              // Display name from SKILL.md
  description: string;
  has_references: boolean;
  has_agents: boolean;
  has_scripts: boolean;
  tools: SkillToolEntry[];   // Which tools have this skill
}

interface SkillToolEntry {
  tool_id: string;           // e.g. "claude", "codex"
  tool_name: string;         // Display name
  dir_path: string;          // Absolute path to this tool's copy
  skill_file_path: string;
  is_symlink: boolean;
  symlink_target: string | null;
  disabled: boolean;
}
```

### SkillContent

```typescript
interface SkillContent {
  frontmatter: Record<string, string>;  // YAML frontmatter key-value pairs
  markdown_body: string;                // Markdown body
  raw_content: string;                  // Raw file content
  references: ReferenceFile[];          // Files under references/
}
```

## Dependencies

### npm Packages

| Package | Purpose |
|---------|---------|
| react-router-dom | Hash Router routing |
| zustand | Lightweight state management |
| react-markdown | Markdown rendering |
| remark-gfm | GitHub Flavored Markdown |
| remark-frontmatter | Skip YAML frontmatter during rendering |
| gray-matter | Parse YAML frontmatter |
| lucide-react | Icon library |

### Rust Crates

| Crate | Purpose |
|-------|---------|
| walkdir | Recursive directory traversal |
| dirs | Cross-platform home directory resolution |
| serde / serde_json | Serialization |

## Running the App

```bash
# Development mode
bun run tauri dev

# Production build
bun run tauri build
```

## Notes

- Creating symlinks on Windows still requires Developer Mode or administrator privileges. When a symlink install fails with the Windows privilege error, the app now automatically retries through a one-shot elevated helper so the user can confirm the UAC prompt and continue.
- Codex skills reside under the `.system/` subdirectory; the registry handles this specially.
- Gemini and Copilot skills directories may be empty; the UI shows an empty state with guidance.
- All file system operations are handled by the Rust backend; the frontend never accesses the file system directly.
