use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};
use windows::{Win32::Foundation::MAX_PATH, Win32::Storage::FileSystem::GetLogicalDriveStringsW};

use crate::debug_println;
use memoize::memoize;

fn build_folder_map(dir: &Path, shared_vector: Arc<Mutex<Vec<PathBuf>>>) {
    // Check if the path is a directory
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                // Recurse into directories
                if path.is_dir() {
                    debug_println!("Scanned: {}", path.display());
                    {
                        let mut vec = shared_vector.lock().unwrap();
                        vec.push(path.clone());
                    }

                    build_folder_map(&path, Arc::clone(&shared_vector));
                }
            }
        }
    }
}

#[memoize]
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
    let drives: Vec<String> = drive_strings
        .clone()
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|str| str.to_string())
        .collect();

    let shared_directories: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::<PathBuf>::new()));

    let mut handles = vec![];

    for drive in drives {
        let shared_vector_clone: Arc<Mutex<Vec<PathBuf>>> = Arc::clone(&shared_directories);
        let root_dir = PathBuf::from(drive);
        let handle = thread::spawn(move || {
            build_folder_map(&root_dir, shared_vector_clone);
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().map_err(|_| "Thread panicked".to_string())?;
    }
    // Return the accumulated results
    let result = Arc::try_unwrap(shared_directories)
        .map_err(|_| "Failed to unwrap Arc".to_string())?
        .into_inner()
        .map_err(|_| "Failed to unlock Mutex".to_string())?;

    println!("{:?}", result);
    Ok(paths)
}
