# Plugin Manager

## Overview

Plugin Manager is a module of the ai-manager desktop application that provides full compatibility with Claude Code's plugin system. It supports adding plugins from local filesystem paths or GitHub repositories, importing from Claude Code marketplace repositories, browsing plugin contents (skills and commands), and installing them to any AI coding tool on the machine — individually or in bulk.

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

### Auto-Detection

The `add_plugin` unified command auto-detects the source type:
- Absolute path (e.g., `D:\plugins\my-plugin`) → treated as Local
- Otherwise (e.g., `owner/repo`) → treated as GitHub

## Marketplace

### Overview

Marketplace support allows batch-importing plugins from a Claude Code compatible marketplace repository. A marketplace is a Git repo (or local directory) containing `.claude-plugin/marketplace.json` that lists multiple plugins.

### marketplace.json Format

```json
{
  "name": "My Plugin Marketplace",
  "owner": { "name": "Owner Name", "email": "owner@example.com" },
  "metadata": {
    "description": "A collection of useful plugins",
    "version": "1.0.0",
    "pluginRoot": "plugins"
  },
  "plugins": [
    {
      "name": "my-plugin",
      "source": "./plugins/my-plugin",
      "description": "Plugin description",
      "version": "1.0.0",
      "author": "Author Name",
      "keywords": ["keyword1"]
    },
    {
      "name": "remote-plugin",
      "source": { "source": "github", "repo": "owner/repo" },
      "description": "A GitHub-hosted plugin"
    }
  ]
}
```

### Plugin Source Types in marketplace.json

The `source` field in each plugin entry supports:

| Type | Format | Description |
|------|--------|-------------|
| Relative path | `"./path/to/plugin"` | Local subdirectory within the marketplace repo |
| GitHub | `{ "source": "github", "repo": "owner/repo" }` | GitHub repository |
| URL | `{ "source": "url", "url": "https://..." }` | Generic Git URL |
| Git subdir | `{ "source": "git-subdir", "url": "...", "path": "..." }` | Subdirectory within a Git repo |
| npm | `{ "source": "npm", "package": "pkg-name" }` | npm package (parsed but not yet supported for import) |

### Marketplace Sources

Like plugins, marketplace sources are auto-detected:
- Absolute path → reads `.claude-plugin/marketplace.json` from local directory
- `owner/repo` or GitHub URL → clones into `~/.agents/plugins/marketplaces/{owner}--{repo}/`

### Marketplace Workflow

1. **Fetch**: User enters a marketplace source (local path or GitHub `owner/repo`). The app clones/reads and parses `marketplace.json`, then shows a preview of available plugins with their source types and whether they are already added.
2. **Import**: Batch-imports all listed plugins. Relative-path plugins are cloned/copied from the marketplace directory; GitHub/URL plugins are cloned independently. Results show per-plugin success/skip/fail status.
3. **Persist**: After import, a `MarketplaceEntry` is saved to the registry so the marketplace can be updated or removed later.
4. **Update**: Re-reads (local) or re-pulls (GitHub) the marketplace, then imports any new plugins and updates metadata for existing ones.
5. **Remove**: Deletes the marketplace entry and all plugins that were imported from it.

### Marketplace Storage

Marketplace references are persisted in the same `registry.json` alongside plugins:

```json
{
  "plugins": [ ... ],
  "marketplaces": [
    {
      "id": "my-marketplace",
      "url": "owner/marketplace-repo",
      "name": "My Plugin Marketplace",
      "owner_name": "Owner Name",
      "plugin_count": 5,
      "added_at": "1710000000"
    }
  ]
}
```

Plugins imported from a marketplace have a `marketplace_id` field linking them back to the marketplace entry.

### Marketplace ID Generation

- Slugified from marketplace name (lowercased, non-alphanumeric characters replaced with hyphens)

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
      "added_at": "1710000000",
      "marketplace_id": null
    }
  ],
  "marketplaces": [
    {
      "id": "my-marketplace",
      "url": "owner/marketplace-repo",
      "name": "My Plugin Marketplace",
      "owner_name": "Owner Name",
      "plugin_count": 5,
      "added_at": "1710000000"
    }
  ]
}
```

### Directory Layout

```
~/.agents/plugins/
├── registry.json              # Persistent plugin list + marketplace entries
├── repos/                     # Cloned GitHub plugin repos
│   └── owner--repo/
│       ├── .claude-plugin/
│       │   └── plugin.json
│       ├── skills/
│       └── commands/
└── marketplaces/              # Cloned marketplace repos
    └── owner--marketplace-repo/
        └── .claude-plugin/
            └── marketplace.json
```

### Plugin ID Generation

- GitHub plugins: `{owner}--{repo}` (e.g., `anthropics--claude-plugins-official`)
- Local plugins: slugified name from `plugin.json` (lowercased, spaces to hyphens, alphanumeric + hyphens only)

## Commands Support

Commands are slash command `.md` files that live in a plugin's `commands/` directory. Each AI tool may have a `commands_subdir` in its tool definition:

| Tool | Commands Directory |
|------|-------------------|
| Claude Code | `~/.claude/commands/` |
| Codex (OpenAI) | `~/.codex/prompts/` |
| Other tools | Not supported by the current Markdown install model (`commands_subdir: None`) |

Command files are symlinked (file symlinks, not directory symlinks) from the plugin source to the tool's command or prompt directory. On Windows, file symlinks require Developer Mode or administrator privileges.

## Installation Mechanism

### Skills

Skills are **directory symlinks** from the plugin's `skills/{skill-name}/` to the target tool's skills directory (e.g., `~/.claude/skills/{skill-name}`).

- Uses the existing `create_skill_symlink` function from `fs_utils.rs`
- Checks for existing installations via canonical path comparison
- Skips already-installed skills during batch operations

### Commands

Commands are **file symlinks** from the plugin's `commands/{name}.md` to the target tool's command or prompt directory.

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
├── models.rs        # Data structs (PluginEntry, PluginSource, MarketplaceEntry, etc.)
├── marketplace.rs   # Marketplace serde models (MarketplaceJson, MarketplacePluginSource, etc.)
├── storage.rs       # Registry persistence (load/save ~/.agents/plugins/registry.json)
├── github.rs        # Git CLI integration (clone, pull, availability check)
└── commands.rs      # 21 Tauri commands
```

### Frontend

```
src/
├── types/plugins.ts                          # TypeScript interfaces
├── api/plugins.ts                            # Tauri invoke wrappers (19 APIs)
├── store/usePluginsStore.ts                  # Zustand store (plugins + marketplaces)
├── hooks/
│   ├── usePlugins.ts                         # Plugin list hook
│   └── usePluginContents.ts                  # Plugin contents hook
├── components/plugins/
│   ├── PluginCard.tsx                        # Plugin list card (source badge, metadata)
│   ├── AddPluginDialog.tsx                   # Add dialog (Plugin / Marketplace tabs)
│   └── PluginInstallDialog.tsx               # Install-to-tool dialog (single + all)
└── pages/plugins/
    ├── PluginsPage.tsx                       # Main plugin list + marketplace section (add/update/remove)
    └── PluginDetailPage.tsx                  # Detail view (skills & commands with install/remove)
```

### Modified Existing Files

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | Added `mod plugins` and registered 21 new commands |
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
| `add_plugin` | `input` | `PluginEntry` | Add plugin (auto-detects local path vs GitHub) |
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
| `install_all_plugin_skills_to_all_tools` | `plugin_id` | `()` | Symlink every plugin skill to every detected skills-capable tool in one batch |
| `install_all_plugin_commands_to_all_tools` | `plugin_id` | `()` | Symlink every plugin command to every detected commands-capable tool in one batch |
| `remove_plugin_skill` | `plugin_id`, `skill_dir_name`, `tool_id` | `()` | Remove skill symlink from tool |
| `remove_plugin_command` | `plugin_id`, `command_file`, `tool_id` | `()` | Remove command symlink from tool |
| `fetch_marketplace` | `url` | `MarketplaceInfo` | Clone/read marketplace and return plugin preview |
| `import_marketplace_plugins` | `url` | `MarketplaceImportResult` | Batch-import all plugins from marketplace |
| `list_marketplaces` | - | `Vec<MarketplaceEntry>` | List saved marketplace entries |
| `update_marketplace` | `marketplace_id` | `MarketplaceImportResult` | Re-fetch marketplace and import new plugins |
| `remove_marketplace` | `marketplace_id` | `()` | Remove marketplace and all its plugins |

## Data Models

### PluginEntry

```typescript
interface PluginEntry {
  id: string;                // Deterministic ID (owner--repo or slugified name)
  source: PluginSource;      // { type: "Local", path } | { type: "GitHub", owner, repo }
  local_path: string;        // Absolute path to plugin root on disk
  metadata: PluginMetadata;  // Parsed from plugin.json
  added_at: string;          // Timestamp
  marketplace_id?: string;   // ID of the marketplace this plugin was imported from (if any)
}
```

### MarketplaceEntry

```typescript
interface MarketplaceEntry {
  id: string;                // Slugified marketplace name
  url: string;               // Original input (owner/repo or local path)
  name: string;              // From marketplace.json
  owner_name: string;        // From marketplace.json
  plugin_count: number;      // Number of plugins in marketplace
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
- File symlinks on Windows require Developer Mode or administrator privileges, same as directory symlinks. If the normal install attempt hits the Windows symlink privilege error, the app automatically retries the plugin install action through a UAC prompt.
- When removing a plugin, already-installed skills/commands remain in their target tools; only the registry entry and cloned repo are removed.
- The `commands_subdir` field is currently set for Claude Code (`commands/`) and Codex (`prompts/`). Other tools can be added only when they expose a compatible Markdown-based target directory.
- Install status detection works by comparing canonical paths of symlinks in tool directories against the plugin source paths.
