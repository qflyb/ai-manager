use crate::skills::registry;
use std::path::PathBuf;

pub(crate) fn collect_tool_commands_dirs() -> Vec<(String, PathBuf)> {
    registry::get_tool_registry()
        .iter()
        .filter_map(|entry| {
            let subdir = entry.def.commands_subdir?;
            let config_dir = (entry.dir_resolver)()?;
            if !config_dir.exists() {
                return None;
            }

            Some((entry.def.name.to_string(), config_dir.join(subdir)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_supports_claude_and_codex_command_targets() {
        let supported_command_tools: Vec<(&str, &str)> = registry::get_tool_registry()
            .into_iter()
            .filter_map(|entry| Some((entry.def.id, entry.def.commands_subdir?)))
            .collect();

        assert!(supported_command_tools.contains(&("claude", "commands")));
        assert!(supported_command_tools.contains(&("codex", "prompts")));
        assert!(!supported_command_tools.contains(&("gemini", "commands")));
        assert!(!supported_command_tools.contains(&("copilot", "agents")));
    }
}
