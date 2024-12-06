use dirs_next;
use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use tauri::{Emitter, Window};

#[derive(Clone, Serialize)]
struct EventPayload {
    name: String,
    total: usize,
    current: usize,
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
pub fn find_games(window: Window) -> Vec<LocationData> {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let json_path = Path::new(cargo_dir).join("data").join("location.json");

    let json_file = File::open(json_path).expect("`location.json` not found");
    let json_reader = BufReader::new(json_file);

    let json: Vec<LocationData> = from_reader(json_reader).expect("JSON was not well-formatted");
    let json_length = json.len();

    let mut found_games: Vec<LocationData> = Vec::new();

    'main: for (index, item) in json.into_iter().enumerate() {
        let parent_directory: PathBuf = match item.parent.as_str() {
            "Document" => dirs_next::document_dir().unwrap(),
            _ => panic!("Parent Directory is invalid"),
        };

        let child_directory = parent_directory.join(item.directory.clone());
        if !child_directory.exists() {
            continue 'main;
        };

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

        window
            .emit(
                "main-loop-progress",
                EventPayload {
                    name: item.name.clone(),
                    total: json_length,
                    current: index,
                },
            )
            .expect("Cannot send `main-loop-progress` event");
        found_games.push(item); // Cloning the item to get an owned value
    }

    found_games
}
