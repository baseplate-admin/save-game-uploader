// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod save_games;
use crate::save_games::find_games;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![find_games])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
