use std::path::PathBuf;

use memoize::memoize;
use windows::Win32::Storage::FileSystem::GetLogicalDrives;

#[memoize]
pub fn build_directory_map() -> Result<Vec<PathBuf>, String> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let logial_drives = unsafe { GetLogicalDrives() };
    println!("{}", logial_drives);
    Ok(paths)
}
