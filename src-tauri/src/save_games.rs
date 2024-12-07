use crate::debug_println;
use dirs_next;
use glob::glob;
use json5;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    // thread,
    // time::Duration,
};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
struct EventPayload {
    name: String,
    total: u64,
    current: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocationData {
    name: String,
    parent: String,
    directory: String,
    globs: Vec<String>,
    image: String,
}
#[tauri::command]
pub async fn find_games(app_handle: AppHandle) -> Vec<LocationData> {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let json_path = Path::new(cargo_dir).join("data").join("location.json5");

    let json_file = File::open(json_path).expect("`location.json5` not found");
    let mut json_buffer = BufReader::new(json_file);
    let mut json_string = String::new();
    json_buffer
        .read_to_string(&mut json_string)
        .expect("Cannot read `json5` file to string");

    let json: Vec<LocationData> = json5::from_str(&json_string)
        .expect("JSON was not well-formatted according to `LocationData`");
    let json_length = json.len();

    let mut found_games: Vec<LocationData> = Vec::new();

    'main: for (index, item) in json.into_iter().enumerate() {
        let parent_directory: PathBuf = match item.parent.as_str() {
            // Public Directory
            "Public_Document" => dirs_next::public_dir().unwrap().join("Documents"),
            // User directories
            "Document" => dirs_next::document_dir().unwrap(),
            "Local" => dirs_next::data_local_dir().unwrap(), // "AppData/Local"
            "Roaming" => dirs_next::data_dir().unwrap(),     // "AppData/Roaming"
            "ProgramData" => Path::new("C:\\ProgramData").to_path_buf(),

            _ => panic!("Parent Directory is invalid"),
        };

        let child_directory = parent_directory.join(item.directory.clone());

        debug_println!(
            "{} {} {}",
            item.name,
            child_directory.display(),
            child_directory.exists()
        );

        if !child_directory.exists() {
            continue 'main;
        };

        // TODO: Make this function call threaded for increased performance?
        for glob_pattern in item.globs.clone() {
            let pattern_path = child_directory.join(glob_pattern.clone());
            let pattern_str = pattern_path.to_str().expect(&format!(
                "Pattern not right. Found {}. Made {}",
                glob_pattern,
                pattern_path.display()
            ));
            let files = glob(pattern_str)
                .map_err(|e| panic!("Glob pattern not right, {} {}", pattern_str, e))
                .unwrap();

            for entry in files {
                if let Err(_) = entry {
                    println!("Glob File not found for {}", item.name);
                    continue 'main;
                }
            }
        }

        app_handle
            .emit_to(
                "updater",
                "main-loop-progress",
                EventPayload {
                    name: item.name.clone(),
                    total: json_length as u64,
                    current: (index + 1) as u64,
                },
            )
            .unwrap();

        // Debug code for testing lode
        // thread::sleep(Duration::from_millis(4000));
        found_games.push(item);
    }

    found_games
}
