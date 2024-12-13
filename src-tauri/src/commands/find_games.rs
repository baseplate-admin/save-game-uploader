use crate::{debug_println, utils};
use glob::glob;

use dirs_next;
use futures::prelude::*;
use json5;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tauri::{AppHandle, Emitter};
use tokio::{sync::Mutex, task};
use utils::globs::given_glob_check_if_file_exists;
use utils::search::check_if_directory_is_in_disk;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocationData {
    name: String,
    parent: String,
    globs: Vec<String>,
    image: String,
}

#[derive(Clone, Serialize)]
struct ThreadEventPayload {
    name: String,
    total: u64,
    current: u64,
}

#[tauri::command]
pub async fn find_games(app_handle: AppHandle) -> Vec<LocationData> {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let json_path = Path::new(cargo_dir).join("data").join("**/*.json5");
    let files = glob(json_path.to_str().unwrap()).expect("Failed to read glob pattern");

    let mut json = Vec::<LocationData>::new();

    for entry in files {
        match entry {
            Ok(path) => {
                if path.file_name().unwrap().to_str().unwrap() != "data.json5" {
                    panic!(
                        "There should only be `data.json5` but found {}",
                        path.file_name().unwrap().to_str().unwrap()
                    );
                };

                let json_file =
                    File::open(path.clone()).expect(&format!("Cannot load {}.", path.display()));

                let mut json_buffer = BufReader::new(json_file);
                let mut json_string = String::new();
                json_buffer
                    .read_to_string(&mut json_string)
                    .expect("Cannot read `json5` file to string");
                let mut _json: Vec<LocationData> = json5::from_str(&json_string).expect(&format!(
                    "{}, JSON was not well-formatted according to `LocationData`",
                    path.display()
                ));

                json.append(&mut _json);
            }
            Err(e) => panic!("Cannot read json5 file, Reason: {}", e),
        }
    }

    let json_length = json.len();
    if json_length == 0 {
        panic!(
            "There is no `data.json5` file in {} directory",
            json_path.display()
        )
    }

    // States
    let found_games = Arc::new(Mutex::new(Vec::new()));
    let current_counter = Arc::new(AtomicU64::new(0));

    let tasks: Vec<_> = json
        .into_iter()
        .map(|item| {
            let app_handle = app_handle.clone();
            let found_games = Arc::clone(&found_games);
            let current_counter = Arc::clone(&current_counter);

            task::spawn(async move {
                'inner: {
                    if item.parent == "GAMEDIR" {
                        if !check_if_directory_is_in_disk(
                            item.globs.clone(),
                            Some(item.name.clone()),
                        )
                        .unwrap()
                        {
                            break 'inner;
                        }
                    } else {
                        let parent_directory: PathBuf = match item.parent.as_str() {
                            "Public_Document" => dirs_next::public_dir().unwrap().join("Documents"),
                            "Document" => dirs_next::document_dir().unwrap(),
                            "Local" => dirs_next::data_local_dir().unwrap(), // "AppData/Local"
                            "Roaming" => dirs_next::data_dir().unwrap(),     // "AppData/Roaming"
                            "ProgramData" => Path::new("C:\\ProgramData").to_path_buf(),
                            "Program Files (x86)" => {
                                Path::new("C:\\Program Files (x86)").to_path_buf()
                            }
                            _ => panic!("Parent Directory is invalid"),
                        };

                        debug_println!(
                            "{} {} {}",
                            item.name,
                            parent_directory.display(),
                            parent_directory.exists()
                        );

                        if !given_glob_check_if_file_exists(
                            item.globs.clone(),
                            parent_directory,
                            Some(item.name.clone()),
                        )
                        .unwrap()
                        {
                            break 'inner;
                        }
                    }
                }

                let current = current_counter.fetch_add(1, Ordering::SeqCst);
                app_handle
                    .emit_to(
                        "updater",
                        "main-loop-progress",
                        ThreadEventPayload {
                            name: item.name.clone(),
                            total: json_length as u64,
                            current: (current + 1) as u64,
                        },
                    )
                    .unwrap();

                found_games.lock().await.push(item);
            })
        })
        .collect();

    futures::future::join_all(tasks).await;

    Arc::try_unwrap(found_games)
        .expect("Multiple references to `found_games` exist")
        .into_inner()
}
