use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};
use windows::{Win32::Foundation::MAX_PATH, Win32::Storage::FileSystem::GetLogicalDriveStringsW};

use crate::debug_println;

fn build_folder_map(dir: &Path, shared_vector: Arc<Mutex<Vec<PathBuf>>>) {
    // Check if the path is a directory
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                // Push the folder or file name to the shared vector
                {
                    let mut vec = shared_vector.lock().unwrap();
                    vec.push(path.clone());
                }

                // Recurse into directories
                if path.is_dir() {
                    build_folder_map(&path, Arc::clone(&shared_vector));
                }
            }
        }
    }
}

pub fn build_directory_map() -> Result<Vec<PathBuf>, String> {
    let paths: Vec<PathBuf> = Vec::new();
    // Buffer to hold drive strings
    let mut buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    // Call GetLogicalDriveStringsW to retrieve logical drives
    let len = unsafe { GetLogicalDriveStringsW(Some(&mut buffer)) };

    if len == 0 {
        debug_println!("Failed to get logical drive strings");
        // return;
    }

    // Convert the buffer to a Rust string and split by null terminators
    let drive_strings = String::from_utf16_lossy(&buffer[..len as usize]);
    let drives: Vec<&str> = drive_strings
        .clone()
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect();

    let shared_directories: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::<PathBuf>::new()));

    let shared_vector_clone = Arc::clone(&shared_directories);
    for drive in drives {
        let root_dir = Path::new(drive);
        let handle = thread::spawn(move || {
            build_folder_map(root_dir, shared_vector_clone);
        });
    }

    Ok(paths)
}
