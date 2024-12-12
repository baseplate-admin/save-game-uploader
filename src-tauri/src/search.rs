use std::path::PathBuf;
use windows::{Win32::Foundation::MAX_PATH, Win32::Storage::FileSystem::GetLogicalDriveStringsW};

use crate::debug_println;

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
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect();

    // Print each drive
    for drive in drives {
        println!("{}", drive);
    }
    Ok(paths)
}
