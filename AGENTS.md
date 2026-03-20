# AI Skills Manager

Desktop app for managing AI coding tool skills across Claude Code, Codex, Gemini, Copilot, and others. Built with Tauri v2 + React 19 + TypeScript + Tailwind CSS 4.

## Development Guide

Full architecture, tech stack, data models, and API reference: `docs/ai-skills-manager.md`

| Scenario | Reference |
|----------|-----------|
| Tauri invoke wrappers | `src/api/skills.ts` |
| Global state (tools, hubSkills) | `src/store/useSkillsStore.ts` |
| TypeScript types | `src/types/skills.ts` |
| Rust command implementations | `src-tauri/src/skills/commands.rs` |
| Tool registry and cross-platform paths | `src-tauri/src/skills/registry.rs` |
| SKILL.md frontmatter parsing | `src-tauri/src/skills/parser.rs` |
| Symlink create/delete/detect | `src-tauri/src/skills/fs_utils.rs` |
| Rust data structs | `src-tauri/src/skills/models.rs` |
| Skill UI components | `src/components/skills/` |
| Route definitions | `src/main.tsx` |

## Core Rules

1. All file system operations must go through Rust Tauri commands; the frontend never accesses the file system directly.
2. Windows symlink creation requires Developer Mode or administrator privileges — surface this to the user on failure.
3. Codex skills live under `.system/` inside the skills directory, not at the root; the registry handles this specially.
4. Disabled skills are toggled by renaming the directory with a `.disabled-` prefix, not by deletion.
5. Use `#[cfg(unix)]` / `#[cfg(windows)]` conditional compilation for all symlink platform differences.
6. Cursor config paths are platform-specific (`%APPDATA%\Cursor\` on Windows, `~/Library/Application Support/Cursor/` on macOS, `~/.config/Cursor/` on Linux).
7. The app uses Hash Router; all routes must be anchored under `/#/`.
8. New Tauri commands must be registered in `src-tauri/src/lib.rs` and wrapped in `src/api/skills.ts`.
9. `installed_in` field on `Skill` is only populated for hub skills; do not rely on it for tool-local skills.
10. Each file must not exceed 1000 lines of code; if a file would exceed this limit, split it into smaller, logically cohesive modules.
