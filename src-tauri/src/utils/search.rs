use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::mpsc;

use crate::{debug_println, utils};
use utils::globs::given_glob_check_if_file_exists;
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::Storage::FileSystem::GetLogicalDriveStringsW;

const AVOID_DIRS: [&str; 2] = [
    "Windows", // C:/Windows
    "AppData",
];

const SYSTEM_DIRS: [&str; 6] = [
    "System32",
    "WinNT",
    "Program Files",
    "Program Files (x86)",
    "ProgramData",
    "$Recycle.Bin",
];

async fn scan_directory(dir: &Path, sender: mpsc::Sender<PathBuf>) -> Result<(), std::io::Error> {
    let mut dir_queue = vec![dir.to_path_buf()];

    while let Some(current_dir) = dir_queue.pop() {
        // Skip system directories and avoid directories
        if let Some(dir_name) = current_dir.file_name().and_then(|n| n.to_str()) {
            if AVOID_DIRS.contains(&dir_name) || SYSTEM_DIRS.contains(&dir_name) {
                continue;
            }
        }

        // Attempt to read directory, handle access errors
        let entries = match fs::read_dir(&current_dir).await {
            Ok(entries) => entries,
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied
                | std::io::ErrorKind::NotFound
                | std::io::ErrorKind::InvalidInput => {
                    debug_println!(
                        "Skipping directory due to access error: {}",
                        current_dir.display()
                    );
                    continue;
                }
                _ => return Err(e),
            },
        };

        let mut stream = entries;
        'inner: while let Some(entry) = match stream.next_entry().await {
            Ok(entry) => entry,
            Err(e) => {
                debug_println!("Error reading entry in {}: {}", current_dir.display(), e);
                break 'inner;
            }
        } {
            let path = entry.path();

            // Check if it's a directory
            if path.is_dir() {
                let filename = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");

                // Skip system and avoid directories
                if !AVOID_DIRS.contains(&filename) && !SYSTEM_DIRS.contains(&filename) {
                    debug_println!("Scanned: {}", path.display());

                    // Send directory path
                    if let Err(_) = sender.send(path.clone()).await {
                        debug_println!("Channel send failed for: {}", path.display());
                        break;
                    }

                    // Queue subdirectory for processing
                    dir_queue.push(path);
                }
            }
        }
    }

    Ok(())
}

async fn build_directory_map() -> Result<Vec<PathBuf>, String> {
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

    // Create a channel to collect directory paths
    let (sender, mut receiver) = mpsc::channel(1000);

    // Spawn tasks for each drive
    let mut handles = Vec::new();
    for drive in drives {
        let root_dir = PathBuf::from(drive);
        let sender_clone = sender.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = scan_directory(&root_dir, sender_clone).await {
                debug_println!("Error scanning drive {}: {}", root_dir.display(), e);
            }
        });
        handles.push(handle);
    }

    // Drop the original sender to allow receiver to complete
    drop(sender);

    // Collect directories from the channel
    let mut directories = Vec::new();
    while let Some(path) = receiver.recv().await {
        directories.push(path);
    }

    // Wait for all drive scanning tasks to complete
    for handle in handles {
        handle.await.map_err(|e| e.to_string())?;
    }

    Ok(directories)
}

pub async fn check_if_directory_is_in_disk(
    globs: Vec<String>,
    name: Option<String>,
) -> Result<bool, String> {
    let directory_map = build_directory_map().await?;

    for item in directory_map {
        let found = given_glob_check_if_file_exists(globs.clone(), item.clone(), name.clone())
            .unwrap_or(false);

        if found {
            return Ok(true);
        }
    }

    Ok(false)
}
