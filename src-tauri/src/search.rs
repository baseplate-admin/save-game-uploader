use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use windows::{Win32::Foundation::MAX_PATH, Win32::Storage::FileSystem::GetLogicalDriveStringsW};

use crate::debug_println;
use memoize::memoize;
use rayon::prelude::*;

fn build_folder_map(dir: &Path, shared_vector: Arc<Mutex<Vec<PathBuf>>>) {
    if let Ok(entries) = fs::read_dir(dir) {
        let entries: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

        // Push directories to the shared vector and spawn threads for subdirectories
        entries
            .par_iter() // Use rayon's parallel iterator
            .for_each(|path| {
                if path.is_dir() {
                    debug_println!("Scanned: {}", path.display());
                    {
                        let mut vec = shared_vector.lock().unwrap();
                        vec.push(path.clone());
                    }
                    // Recursive call in parallel for subdirectories
                    build_folder_map(path, Arc::clone(&shared_vector));
                }
            });
    }
}

#[memoize]
fn build_directory_map() -> Result<Vec<PathBuf>, String> {
    // Buffer to hold drive strings
    let mut buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    // Call GetLogicalDriveStringsW to retrieve logical drives
    let len = unsafe { GetLogicalDriveStringsW(Some(&mut buffer)) };

    if len == 0 {
        debug_println!("Failed to get logical drive strings");
        return Err("Failed to get logical drive strings".to_string());
    }

    // Convert the buffer to a Rust string and split by null terminators
    let drive_strings = String::from_utf16_lossy(&buffer[..len as usize]);
    let drives: Vec<String> = drive_strings
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let shared_directories: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::<PathBuf>::new()));

    drives.par_iter().for_each(|drive| {
        let shared_vector_clone = Arc::clone(&shared_directories);
        let root_dir = PathBuf::from(drive);
        build_folder_map(&root_dir, shared_vector_clone);
    });

    // Return the accumulated results
    let result = Arc::try_unwrap(shared_directories)
        .map_err(|_| "Failed to unwrap Arc".to_string())?
        .into_inner()
        .map_err(|_| "Failed to unlock Mutex".to_string())?;

    println!("{:?}", result);
    Ok(result)
}

pub fn check_if_directory_is_in_disk(directory: String) -> Result<bool, String> {
    let directory_map = build_directory_map().unwrap();
    Ok(directory_map.iter().any(|path| {
        path.file_name()
            .and_then(|file_name| file_name.to_str())
            .map(|file_name| file_name == directory)
            .unwrap_or(false)
    }))
}
