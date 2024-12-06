use dirs_next;
use glob::glob;
use serde::Deserialize;
use serde_json::from_reader;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct LocationData {
    name: String,
    parent: String,
    directory: String,
    globs: Vec<String>,
}

pub fn find_games() -> Result<Vec<LocationData>, String> {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let json_path = Path::new(cargo_dir).join("data").join("location.json");

    let json_file = File::open(json_path).expect("`location.json` not found");
    let json_reader = BufReader::new(json_file);

    let json: Vec<LocationData> = from_reader(json_reader).expect("JSON was not well-formatted");

    let mut found_games: Vec<LocationData> = Vec::new();

    'main: for item in json {
        let parent_directory: PathBuf;
        match item.parent.as_str() {
            "Document" => parent_directory = dirs_next::document_dir().unwrap(),
            _ => {
                panic!("Parent Directory is invalid");
            }
        }

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
                match entry {
                    Err(_) => {
                        println!("Glob File not found for {}", item.name);
                        continue 'main;
                    }
                    _ => {} // Found entry. Don't do anything
                }
            }
        }
        found_games.push(item);
    }

    Ok(found_games)
}
