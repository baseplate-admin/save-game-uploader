use futures::{stream, StreamExt};
use glob::glob;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

use crate::debug_println;

pub async fn given_glob_check_if_file_exists(
    globs: Vec<String>,
    parent_dir: PathBuf,
    name: Option<String>,
) -> Result<bool, String> {
    let found = Arc::new(Mutex::new(false));

    let tasks = globs.clone().into_iter().map(|glob_pattern| {
        let found = Arc::clone(&found);
        let parent_dir = parent_dir.clone();
        let name = name.clone();

        tokio::spawn(async move {
            let pattern_path = parent_dir.join(glob_pattern.clone());
            let pattern_str = match pattern_path.to_str() {
                Some(s) => s.to_string(),
                None => {
                    debug_println!(
                        "Pattern not right. Found {}. Made {}",
                        glob_pattern,
                        pattern_path.display()
                    );
                    return;
                }
            };

            let files = match glob(&pattern_str) {
                Ok(paths) => paths,
                Err(e) => {
                    debug_println!("Glob pattern not right: {}. Error: {}", pattern_str, e);
                    return;
                }
            };

            for entry in files {
                match entry {
                    Ok(_) => {
                        let mut found_guard = found.lock().await;
                        *found_guard = true;
                        return;
                    }
                    Err(_) => {
                        if let Some(name) = &name {
                            debug_println!("Glob file not found for {}", name);
                        }
                    }
                }
            }
        })
    });

    // Execute all tasks concurrently
    let _: Vec<_> = stream::iter(tasks)
        .buffer_unordered(globs.len())
        .collect()
        .await;

    let found_result = *found.lock().await;
    Ok(found_result)
}
