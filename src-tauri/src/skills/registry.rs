use std::path::PathBuf;

pub struct ToolDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub capability: &'static str,
    pub skills_subdir: Option<&'static str>,
    pub commands_subdir: Option<&'static str>,
    pub config_files: &'static [(&'static str, &'static str)],
}

pub struct ToolEntry {
    pub def: ToolDefinition,
    pub dir_resolver: fn() -> Option<PathBuf>,
}

fn home_relative(dir_name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(dir_name))
}

fn cursor_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .ok()
            .map(|p| PathBuf::from(p).join("Cursor"))
    }
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|h| h.join("Library/Application Support/Cursor"))
    }
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir().map(|h| h.join(".config/Cursor"))
    }
}

pub fn get_tool_registry() -> Vec<ToolEntry> {
    vec![
        // Tools with skills system
        ToolEntry {
            def: ToolDefinition {
                id: "claude",
                name: "Claude Code",
                capability: "skills",
                skills_subdir: Some("skills"),
                commands_subdir: Some("commands"),
                config_files: &[("settings.json", "json"), ("CLAUDE.md", "markdown")],
            },
            dir_resolver: || home_relative(".claude"),
        },
        ToolEntry {
            def: ToolDefinition {
                id: "codex",
                name: "Codex (OpenAI)",
                capability: "skills",
                skills_subdir: Some("skills"),
                commands_subdir: None,
                config_files: &[("config.toml", "toml"), ("AGENTS.md", "markdown")],
            },
            dir_resolver: || home_relative(".codex"),
        },
        ToolEntry {
            def: ToolDefinition {
                id: "gemini",
                name: "Google Gemini",
                capability: "skills",
                skills_subdir: Some("skills"),
                commands_subdir: None,
                config_files: &[("settings.json", "json"), ("GEMINI.md", "markdown")],
            },
            dir_resolver: || home_relative(".gemini"),
        },
        ToolEntry {
            def: ToolDefinition {
                id: "copilot",
                name: "GitHub Copilot",
                capability: "skills",
                skills_subdir: Some("skills"),
                commands_subdir: None,
                config_files: &[],
            },
            dir_resolver: || home_relative(".copilot"),
        },
        // Config-only tools
        ToolEntry {
            def: ToolDefinition {
                id: "cursor",
                name: "Cursor",
                capability: "config-only",
                skills_subdir: None,
                commands_subdir: None,
                config_files: &[],
            },
            dir_resolver: cursor_config_dir,
        },
        ToolEntry {
            def: ToolDefinition {
                id: "codebuddy",
                name: "CodeBuddy",
                capability: "config-only",
                skills_subdir: None,
                commands_subdir: None,
                config_files: &[("mcp.json", "json"), ("argv.json", "json")],
            },
            dir_resolver: || home_relative(".codebuddy"),
        },
        ToolEntry {
            def: ToolDefinition {
                id: "codegeex",
                name: "CodeGeex",
                capability: "config-only",
                skills_subdir: None,
                commands_subdir: None,
                config_files: &[],
            },
            dir_resolver: || home_relative(".codegeex"),
        },
        ToolEntry {
            def: ToolDefinition {
                id: "marscode",
                name: "MarsCode",
                capability: "config-only",
                skills_subdir: None,
                commands_subdir: None,
                config_files: &[("config.json", "json")],
            },
            dir_resolver: || home_relative(".marscode"),
        },
        ToolEntry {
            def: ToolDefinition {
                id: "lingma",
                name: "Lingma",
                capability: "config-only",
                skills_subdir: None,
                commands_subdir: None,
                config_files: &[("lingma_mcp.json", "json")],
            },
            dir_resolver: || home_relative(".lingma"),
        },
        ToolEntry {
            def: ToolDefinition {
                id: "kiro",
                name: "Kiro",
                capability: "config-only",
                skills_subdir: None,
                commands_subdir: None,
                config_files: &[],
            },
            dir_resolver: || home_relative(".kiro"),
        },
        ToolEntry {
            def: ToolDefinition {
                id: "gongfeng_copilot",
                name: "Gongfeng Copilot",
                capability: "config-only",
                skills_subdir: None,
                commands_subdir: None,
                config_files: &[("config.json", "json")],
            },
            dir_resolver: || home_relative(".gongfeng_copilot"),
        },
        ToolEntry {
            def: ToolDefinition {
                id: "coding_copilot",
                name: "CodingCopilot",
                capability: "config-only",
                skills_subdir: None,
                commands_subdir: None,
                config_files: &[],
            },
            dir_resolver: || home_relative(".codingCopilot"),
        },
    ]
}

pub fn get_hub_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".agents").join("skills"))
}
