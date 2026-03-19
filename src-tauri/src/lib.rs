mod skills;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            skills::commands::scan_ai_tools,
            skills::commands::list_skills,
            skills::commands::read_skill,
            skills::commands::get_hub_skills,
            skills::commands::install_skill,
            skills::commands::remove_skill,
            skills::commands::toggle_skill,
            skills::commands::read_config_file,
            skills::commands::detect_editors,
            skills::commands::open_in_editor,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
