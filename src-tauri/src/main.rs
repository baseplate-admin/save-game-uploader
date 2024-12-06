// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod save_games;
use crate::save_games::find_games;
fn main() {
    // save_game_uploader_lib::run();
    find_games().unwrap();
}
