// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod macros;
mod save_games;
mod search;
mod utils;

use save_games::find_games;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![find_games])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
