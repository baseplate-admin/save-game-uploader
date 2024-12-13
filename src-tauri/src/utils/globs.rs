use futures::{stream, StreamExt};
use glob::glob;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub async fn given_glob_check_if_file_exists(
    globs: Vec<String>,
    parent_dir: PathBuf,
    name: Option<String>,
) -> Result<bool, String> {
    let found = Arc::new(AtomicBool::new(false));

    let tasks = globs.clone().into_iter().map(|glob_pattern| {
        let found = Arc::clone(&found);
        let parent_dir = parent_dir.clone();
        let name = name.clone();

        tokio::spawn(async move {
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
                match entry {
                    Ok(_) => {
                        found.store(true, Ordering::SeqCst);
                    }
                    Err(_) => {
                        // We are going to invalidate if we dont find anything that matches glob
                        found.store(false, Ordering::SeqCst);

                        if let Some(_name) = &name {
                            println!("Glob File not found for {}", _name);
                        }
                    }
                }
            }
        })
    });
    let _: Vec<_> = stream::iter(tasks)
        .buffer_unordered(globs.len())
        .collect()
        .await;

    Ok(found.load(Ordering::SeqCst))
}
