// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // save_game_uploader_lib::run();
    save_game_uploader_lib::find_games();
}
