use crate::advisor::models::*;
use std::path::Path;

pub fn heuristic_analysis(
    base: &Path,
    min_size: u64,
    older_than_days: Option<u64>,
) -> Vec<Recommendation> {
    let mut recs = Vec::new();
    let threshold_days = older_than_days.unwrap_or(90);

    // Scan Movies for large old files
    let movies = base.join("Movies");
    if movies.exists() {
        scan_large_old_files(&movies, min_size, threshold_days, &mut recs);
    }

    // Scan Desktop for large old files
    let desktop = base.join("Desktop");
    if desktop.exists() {
        scan_large_old_files(&desktop, min_size, threshold_days, &mut recs);
    }

    recs
}

fn scan_large_old_files(
    dir: &Path,
    min_size: u64,
    threshold_days: u64,
    recs: &mut Vec<Recommendation>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if !metadata.is_file() {
            continue;
        }
        if metadata.len() < min_size {
            continue;
        }

        let days_since_access = match metadata.accessed() {
            Ok(accessed) => match accessed.elapsed() {
                Ok(elapsed) => elapsed.as_secs() / 86400,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if days_since_access > threshold_days {
            recs.push(Recommendation {
                category: Category::UserOldFiles,
                path: path.display().to_string(),
                size: metadata.len(),
                risk: Risk::Review,
                reason: format!(
                    "Large file not accessed in {} days: {}",
                    days_since_access,
                    human_bytes(metadata.len())
                ),
                suggested_command: format!("ls -lh '{}'", path.display()),
                last_accessed_days: Some(days_since_access),
            });
        }
    }
}
