# Plugin Manager

## Overview

Plugin Manager is a module of the ai-manager desktop application that provides full compatibility with Claude Code's plugin system. It supports adding plugins from local filesystem paths or GitHub repositories, browsing plugin contents (skills and commands), and installing them to any AI coding tool on the machine — individually or in bulk.

## Plugin Structure (Claude Code Compatible)

```
plugin-name/
├── .claude-plugin/
│   └── plugin.json          # Plugin metadata (required)
├── skills/                  # Skill directories (optional)
│   └── skill-name/
│       └── SKILL.md
├── commands/                # Slash command .md files (optional)
│   └── command-name.md
└── README.md
```

### plugin.json Format

```json
{
  "name": "my-plugin",
  "description": "Plugin description",
  "version": "1.0.0",
  "author": { "name": "Author Name", "email": "author@example.com" },
  "license": "MIT",
  "keywords": ["keyword1", "keyword2"],
  "repository": "https://github.com/owner/repo"
}
```

The `author` field supports both object form `{ "name": "...", "email": "..." }` and plain string `"Author Name"`.

## Plugin Sources

### Local Plugins

- Added by specifying an absolute filesystem path (e.g., `D:\plugins\my-plugin`)
- The path is stored as-is in the registry; no files are copied
- The plugin directory must contain `.claude-plugin/plugin.json`

### GitHub Plugins

- Added by specifying `owner/repo` format or a full GitHub URL
- Cloned via `git clone --depth 1` into `~/.agents/plugins/repos/{owner}--{repo}/`
- Requires Git to be installed and available in PATH
- Can be updated (re-pulled) from the UI

## Storage

### Registry

Location: `~/.agents/plugins/registry.json`

```json
{
  "plugins": [
    {
      "id": "owner--repo",
      "source": { "type": "GitHub", "owner": "owner", "repo": "repo" },
      "local_path": "/home/user/.agents/plugins/repos/owner--repo",
      "metadata": {
        "name": "Plugin Name",
        "description": "...",
        "version": "1.0.0",
        "author": "Author",
        "license": "MIT",
        "keywords": ["..."],
        "repository": "..."
      },
      "added_at": "1710000000"
    }
  ]
}
```

### Directory Layout

```
~/.agents/plugins/
├── registry.json              # Persistent plugin list
└── repos/                     # Cloned GitHub plugin repos
    └── owner--repo/
        ├── .claude-plugin/
        │   └── plugin.json
        ├── skills/
        └── commands/
```

### Plugin ID Generation

- GitHub plugins: `{owner}--{repo}` (e.g., `anthropics--claude-plugins-official`)
- Local plugins: slugified name from `plugin.json` (lowercased, spaces to hyphens, alphanumeric + hyphens only)

## Commands Support

Commands are slash command `.md` files that live in a plugin's `commands/` directory. Each AI tool may have a `commands_subdir` in its tool definition:

| Tool | Commands Directory |
|------|-------------------|
| Claude Code | `~/.claude/commands/` |
| Other tools | Not supported yet (`commands_subdir: None`) |

Command files are symlinked (file symlinks, not directory symlinks) from the plugin source to the tool's commands directory. On Windows, file symlinks require Developer Mode or administrator privileges.

## Installation Mechanism

### Skills

Skills are **directory symlinks** from the plugin's `skills/{skill-name}/` to the target tool's skills directory (e.g., `~/.claude/skills/{skill-name}`).

- Uses the existing `create_skill_symlink` function from `fs_utils.rs`
- Checks for existing installations via canonical path comparison
- Skips already-installed skills during batch operations

### Commands

Commands are **file symlinks** from the plugin's `commands/{name}.md` to the target tool's commands directory.

- Uses the new `create_file_symlink` function in `fs_utils.rs`
- Windows uses `symlink_file()`, Unix uses `symlink()`

### Two Installation Modes

1. **Single tool**: Install a skill or command to one specific AI tool
2. **All tools**: One-click batch install to all detected tools that support the feature

## Project Structure

### Rust Backend

```
src-tauri/src/plugins/
├── mod.rs           # Module exports
├── models.rs        # Data structs (PluginEntry, PluginSource, PluginContents, etc.)
├── storage.rs       # Registry persistence (load/save ~/.agents/plugins/registry.json)
├── github.rs        # Git CLI integration (clone, pull, availability check)
└── commands.rs      # 12 Tauri commands
```

### Frontend

```
src/
├── types/plugins.ts                          # TypeScript interfaces
├── api/plugins.ts                            # Tauri invoke wrappers (12 APIs)
├── store/usePluginsStore.ts                  # Zustand store
├── hooks/
│   ├── usePlugins.ts                         # Plugin list hook
│   └── usePluginContents.ts                  # Plugin contents hook
├── components/plugins/
│   ├── PluginCard.tsx                        # Plugin list card (source badge, metadata)
│   ├── AddPluginDialog.tsx                   # Add dialog (Local Path / GitHub tabs)
│   └── PluginInstallDialog.tsx               # Install-to-tool dialog (single + all)
└── pages/plugins/
    ├── PluginsPage.tsx                       # Main plugin list (add/update/remove)
    └── PluginDetailPage.tsx                  # Detail view (skills & commands with install/remove)
```

### Modified Existing Files

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | Added `mod plugins` and registered 12 new commands |
| `src-tauri/src/skills/registry.rs` | Added `commands_subdir` field to `ToolDefinition` |
| `src-tauri/src/skills/fs_utils.rs` | Added `create_file_symlink` and `remove_file_or_symlink` |
| `src/main.tsx` | Added `/plugins` and `/plugins/:pluginId` routes |
| `src/components/layout/Sidebar.tsx` | Added Plugins navigation entry (Puzzle icon) |
| `src/pages/Home.tsx` | Added Plugins feature card |

## Route Structure

```
/plugins                     → Plugin list (add, update, remove)
/plugins/:pluginId           → Plugin detail (skills & commands, install/remove)
```

## Tauri Commands

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `add_plugin_local` | `path` | `PluginEntry` | Add plugin from local filesystem path |
| `add_plugin_github` | `owner`, `repo` | `PluginEntry` | Clone plugin from GitHub and add |
| `list_plugins` | - | `Vec<PluginEntry>` | List all added plugins |
| `remove_plugin` | `plugin_id` | `()` | Remove plugin (deletes cloned repo for GitHub) |
| `update_plugin` | `plugin_id` | `PluginEntry` | Update plugin (git pull for GitHub, re-read for local) |
| `list_plugin_contents` | `plugin_id` | `PluginContents` | Scan skills/ and commands/ with install status |
| `install_plugin_skill` | `plugin_id`, `skill_dir_name`, `tool_id` | `()` | Symlink skill to specific tool |
| `install_plugin_skill_to_all` | `plugin_id`, `skill_dir_name` | `()` | Symlink skill to all skills-capable tools |
| `install_plugin_command` | `plugin_id`, `command_file`, `tool_id` | `()` | Symlink command to specific tool |
| `install_plugin_command_to_all` | `plugin_id`, `command_file` | `()` | Symlink command to all commands-capable tools |
| `remove_plugin_skill` | `plugin_id`, `skill_dir_name`, `tool_id` | `()` | Remove skill symlink from tool |
| `remove_plugin_command` | `plugin_id`, `command_file`, `tool_id` | `()` | Remove command symlink from tool |

## Data Models

### PluginEntry

```typescript
interface PluginEntry {
  id: string;                // Deterministic ID (owner--repo or slugified name)
  source: PluginSource;      // { type: "Local", path } | { type: "GitHub", owner, repo }
  local_path: string;        // Absolute path to plugin root on disk
  metadata: PluginMetadata;  // Parsed from plugin.json
  added_at: string;          // Timestamp
}
```

### PluginMetadata

```typescript
interface PluginMetadata {
  name: string;
  description: string;
  version: string;
  author: string | null;
  license: string | null;
  keywords: string[] | null;
  repository: string | null;
}
```

### PluginContents

```typescript
interface PluginContents {
  skills: PluginSkillInfo[];    // Skills found in skills/ directory
  commands: PluginCommandInfo[]; // Commands found in commands/ directory
}

interface PluginSkillInfo {
  dir_name: string;         // Skill directory name
  name: string;             // From SKILL.md frontmatter
  description: string;      // From SKILL.md frontmatter
  skill_path: string;       // Absolute path
  installed_in: string[];   // Tool IDs where currently symlinked
}

interface PluginCommandInfo {
  file_name: string;        // e.g., "review.md"
  command_name: string;     // Stem: "review"
  file_path: string;        // Absolute path
  installed_in: string[];   // Tool IDs where currently symlinked
}
```

## Notes

- Git must be installed for GitHub plugin support; the app checks availability and shows a clear error if missing.
- File symlinks on Windows require Developer Mode or administrator privileges, same as directory symlinks.
- When removing a plugin, already-installed skills/commands remain in their target tools; only the registry entry and cloned repo are removed.
- The `commands_subdir` field is currently only set for Claude Code; other tools can be extended as they add commands support.
- Install status detection works by comparing canonical paths of symlinks in tool directories against the plugin source paths.
