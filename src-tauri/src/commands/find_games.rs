use crate::{debug_println, utils};
use glob::glob;

use dirs_next;
use json5;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    }, // thread,
       // time::Duration,
};
use tauri::{AppHandle, Emitter};
use utils::globs::given_glob_check_if_file_exists;
use utils::search::check_if_directory_is_in_disk;

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
    let json_path = Path::new(cargo_dir).join("data").join("**/*.json5");
    let files = glob(json_path.to_str().unwrap()).unwrap();

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
                    File::open(path.clone()).expect(&format!("Canonot load {}.", path.display()));

                let mut json_buffer = BufReader::new(json_file);
                let mut json_string = String::new();
                json_buffer
                    .read_to_string(&mut json_string)
                    .expect("Cannot read `json5` file to string");
                let mut _json: Vec<LocationData> = json5::from_str(&json_string).expect(&format!(
                    "{},JSON was not well-formatted according to `LocationData`",
                    path.display()
                ));

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

    // States
    let found_games: Arc<Mutex<Vec<LocationData>>> = Arc::new(Mutex::new(Vec::new()));
    let current_counter = Arc::new(AtomicU64::new(0));

    json.into_par_iter().for_each(|item| {
        if item.parent == "GAMEDIR" {
            if !check_if_directory_is_in_disk(item.globs.clone(), Option::Some(item.name.clone()))
                .unwrap()
            {
                return;
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

            if !given_glob_check_if_file_exists(
                item.globs.clone(),
                parent_directory,
                Option::Some(item.name.clone()),
            )
            .unwrap()
            {
                return;
            }
        }

        let current = current_counter.load(Ordering::SeqCst);
        current_counter.store(current + 1, Ordering::SeqCst);

        app_handle
            .emit_to(
                "updater",
                "main-loop-progress",
                EventPayload {
                    name: item.name.clone(),
                    total: json_length as u64,
                    current: (current + 1) as u64,
                },
            )
            .unwrap();
        found_games.lock().unwrap().push(item);
    });

    Arc::try_unwrap(found_games)
        .expect("Multiple references to `found_games` exist")
        .into_inner()
        .unwrap()
}
