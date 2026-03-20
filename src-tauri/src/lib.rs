mod cache;
mod plugins;
mod skills;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            skills::commands::scan_ai_tools,
            skills::commands::list_skills,
            skills::commands::list_all_skills,
            skills::commands::read_skill,
            skills::commands::get_hub_skills,
            skills::commands::install_skill,
            skills::commands::remove_skill,
            skills::commands::remove_skill_from_all,
            skills::commands::toggle_skill,
            skills::commands::read_config_file,
            skills::commands::detect_editors,
            skills::commands::open_in_editor,
            cache::commands::get_cache_info,
            cache::commands::clear_tool_cache,
            cache::commands::clear_all_caches,
            plugins::commands::add_plugin_local,
            plugins::commands::add_plugin_github,
            plugins::commands::list_plugins,
            plugins::commands::remove_plugin,
            plugins::commands::update_plugin,
            plugins::commands::list_plugin_contents,
            plugins::commands::install_plugin_skill,
            plugins::commands::install_plugin_skill_to_all,
            plugins::commands::install_plugin_command,
            plugins::commands::install_plugin_command_to_all,
            plugins::commands::remove_plugin_skill,
            plugins::commands::remove_plugin_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
