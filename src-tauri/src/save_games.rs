use crate::{debug_println, search::check_if_directory_is_in_disk, utils};
use glob::glob;

use dirs_next;
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
use utils::globs::given_glob_check_if_file_exists;

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
    globs: Vec<String>,
    image: String,
}

#[tauri::command]
pub async fn find_games(app_handle: AppHandle) -> Vec<LocationData> {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let json_path = Path::new(cargo_dir).join("data").join("**/*.json");
    let files = glob(json_path.to_str().unwrap()).unwrap();

    let mut json = Vec::<LocationData>::new();

    for entry in files {
        match entry {
            Ok(ref path) => {
                println!("{:#?}", path);

                let json_file =
                    File::open(path.clone()).expect(&format!("Canonot load {}.", path.display()));

                let mut json_buffer = BufReader::new(json_file);
                let mut json_string = String::new();
                json_buffer
                    .read_to_string(&mut json_string)
                    .expect("Cannot read `json5` file to string");
                let mut _json: Vec<LocationData> = json5::from_str(&json_string)
                    .expect("JSON was not well-formatted according to `LocationData`");

                json.append(&mut _json);
            }
            Err(e) => panic!("Cannot read json5 file, Reson {}", e),
        }
    }

    let json_length = json.len();
    if json_length == 0 {
        panic!(
            "There is no `data.json5` file in {} directory",
            json_path.display()
        )
    }

    let mut found_games: Vec<LocationData> = Vec::new();

    for (index, item) in json.into_iter().enumerate() {
        if item.parent == "<GAMEDIR>" {
            if check_if_directory_is_in_disk(item.globs.clone(), Option::Some(item.name.clone()))
                .unwrap()
            {
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
                found_games.push(item);
            }
        } else {
            let parent_directory: PathBuf = match item.parent.as_str() {
                // Public Directory
                "Public_Document" => dirs_next::public_dir().unwrap().join("Documents"),
                // User directories
                "Document" => dirs_next::document_dir().unwrap(),
                "Local" => dirs_next::data_local_dir().unwrap(), // "AppData/Local"
                "Roaming" => dirs_next::data_dir().unwrap(),     // "AppData/Roaming"
                "ProgramData" => Path::new("C:\\ProgramData").to_path_buf(),
                "Program Files (x86)" => Path::new("C:\\Program Files (x86)").to_path_buf(),

                _ => panic!("Parent Directory is invalid"),
            };

            debug_println!(
                "{} {} {}",
                item.name,
                parent_directory.display(),
                parent_directory.exists()
            );

            let directory_is_found = given_glob_check_if_file_exists(
                item.globs.clone(),
                parent_directory,
                Option::Some(item.name.clone()),
            )
            .unwrap();
            if directory_is_found {
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
                found_games.push(item);
            }
        }

        // Debug code for testing lode
        // thread::sleep(Duration::from_millis(4000));
    }

    found_games
}
