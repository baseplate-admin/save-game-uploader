use glob::glob;
use rayon::prelude::*;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub fn given_glob_check_if_file_exists(
    globs: Vec<String>,
    parent_dir: PathBuf,
    name: Option<String>,
) -> Result<bool, String> {
    let found = Arc::new(AtomicBool::new(false));

    globs.par_iter().for_each(|glob_pattern| {
        // Early exit if found is true
        if found.load(Ordering::SeqCst) {
            return;
        }

        let pattern_path = parent_dir.join(glob_pattern.clone());
        let pattern_str = pattern_path.to_str().expect(&format!(
            "Pattern not right. Found {}. Made {}",
            glob_pattern,
            pattern_path.display()
        ));
        let files = glob(pattern_str)
            .map_err(|e| panic!("Glob pattern not right, {} {}", pattern_str, e))
            .unwrap();

        for entry in files {
            if let Err(_) = entry {
                if let Some(_name) = &name {
                    println!("Glob File not found for {}", _name);
                }
            } else {
                found.store(true, Ordering::SeqCst);
            }
        }
    });
    Ok(found.load(Ordering::SeqCst))
}
