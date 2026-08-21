use anyhow::Result;

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

const CACHE_DIR: &str = ".prep-cache";

pub fn history(clear: bool, verbose: bool) -> Result<()> {
    if clear {
        clear_cache();
        return Ok(());
    }

    if !Path::new(CACHE_DIR).exists() {
        println!("No cache found.");
        return Ok(());
    }

    let entries = fs::read_dir(CACHE_DIR)?;
    let mut count = 0;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            count += 1;
            if verbose {
                let metadata = fs::metadata(&path)?;
                let modified = metadata.modified()?;
                let duration = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
                let time = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                println!(
                    "{} - {}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    time
                );
            }
        }
    }
    if !verbose {
        println!("{} cache entries found.", count);
    }

    Ok(())
}

pub fn clean_cache(all: bool, dry_run: bool) -> Result<()> {
    if all {
        clean_all_cache(dry_run)
    } else {
        clean_temp_files(dry_run)
    }
}

fn clear_cache() {
    if Path::new(CACHE_DIR).exists() {
        let _ = fs::remove_dir_all(CACHE_DIR);
        println!("Cache cleared.");
    } else {
        println!("No cache to clear.");
    }
}

fn clean_all_cache(dry_run: bool) -> Result<()> {
    if dry_run {
        if Path::new(CACHE_DIR).exists() {
            let entries = fs::read_dir(CACHE_DIR)?;
            for entry in entries {
                let entry = entry?;
                println!("Would remove: {}", entry.path().display());
            }
        }
        return Ok(());
    }

    clear_cache();
    // Also clean other temp files
    clean_temp_files(false)
}

fn clean_temp_files(dry_run: bool) -> Result<()> {
    let patterns = ["*.tmp", "*.log", "*.cache"];
    for pattern in patterns {
        let entries = glob::glob(pattern)?;
        for entry in entries {
            if let Ok(path) = entry {
                if dry_run {
                    println!("Would remove: {}", path.display());
                } else {
                    if path.is_file() {
                        fs::remove_file(&path)?;
                        println!("Removed: {}", path.display());
                    }
                }
            }
        }
    }
    Ok(())
}
