use crate::advisor::models::{dir_size, Category, FolderSummary, Recommendation, Risk, TopFile};
use std::path::Path;
use std::time::SystemTime;

pub fn scan_user_folders(home: &Path) -> Vec<FolderSummary> {
    let mut folders = Vec::new();

    // Scan standard user folders
    let folder_names = if cfg!(target_os = "macos") {
        vec!["Downloads", "Documents", "Desktop", "Movies", "Music", "Pictures"]
    } else {
        vec!["Downloads", "Documents", "Desktop", "Videos", "Music", "Pictures"]
    };

    for name in folder_names {
        let path = home.join(name);
        if path.exists() && path.is_dir() {
            if let Some(summary) = scan_single_folder(&path, name) {
                folders.push(summary);
            }
        }
    }

    // Scan hidden directories (starting with .) in home
    if let Ok(entries) = std::fs::read_dir(home) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            
            // Skip non-hidden and special dirs
            if !name.starts_with('.') || name == "." || name == ".." {
                continue;
            }

            // Skip common system/config dirs we already handle
            let skip_dirs = [
                ".Trash", ".git", ".qwen", ".vscode", ".idea", ".DS_Store",
            ];
            if skip_dirs.contains(&name) {
                continue;
            }

            if let Some(summary) = scan_single_folder(&path, name) {
                if summary.total_size > 0 {
                    folders.push(summary);
                }
            }
        }
    }

    folders.sort_by_key(|f| std::cmp::Reverse(f.total_size));
    folders
}

pub fn scan_home_summary(home: &Path) -> Option<FolderSummary> {
    scan_single_folder(home, "Home (~)")
}

pub fn scan_user_folders_as_recommendations(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();
    let folders = scan_user_folders(home);

    for folder in folders {
        if folder.old_files_90d > 0 {
            let estimated_old_size = if folder.file_count > 0 {
                (folder.total_size / folder.file_count) * folder.old_files_90d
            } else {
                0
            };

            if estimated_old_size >= min_size {
                recs.push(Recommendation {
                    category: Category::UserOldFiles,
                    path: folder.path.display().to_string(),
                    size: estimated_old_size,
                    risk: Risk::Review,
                    reason: format!(
                        "{} has {} files not accessed in 90+ days (~{})",
                        folder.name,
                        folder.old_files_90d,
                        crate::advisor::models::human_bytes(estimated_old_size)
                    ),
                    suggested_command: format!("ls -lht '{}'", folder.path.display()),
                    last_accessed_days: Some(90),
                });
            }
        }

        for top_file in folder.top_files.iter().take(5) {
            if top_file.size >= min_size {
                let days_ago = top_file.last_accessed_days.unwrap_or(0);
                recs.push(Recommendation {
                    category: Category::UserOldFiles,
                    path: top_file.path.display().to_string(),
                    size: top_file.size,
                    risk: Risk::Review,
                    reason: format!(
                        "Large file in {}: {} (not accessed in {} days)",
                        folder.name, top_file.name, days_ago
                    ),
                    suggested_command: format!("ls -lh '{}'", top_file.path.display()),
                    last_accessed_days: top_file.last_accessed_days,
                });
            }
        }
    }

    recs
}

fn scan_single_folder(path: &Path, name: &str) -> Option<FolderSummary> {
    let total_size = dir_size(path).unwrap_or(0);

    let mut file_count = 0u64;
    let mut old_files_90d = 0u64;
    let mut old_files_180d = 0u64;
    let mut old_files_365d = 0u64;
    let mut all_files: Vec<TopFile> = Vec::new();
    let mut subfolder_sizes: Vec<(String, u64)> = Vec::new();

    let now = SystemTime::now();

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return None,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_dir() && !entry_path.is_symlink() {
            let sub_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let sub_size = dir_size(&entry_path).unwrap_or(0);
            if sub_size > 0 {
                subfolder_sizes.push((sub_name, sub_size));
            }

            let _ = scan_files_recursive(
                &entry_path,
                &now,
                &mut file_count,
                &mut old_files_90d,
                &mut old_files_180d,
                &mut old_files_365d,
                &mut all_files,
            );
        } else if metadata.is_file() {
            file_count += 1;
            let size = metadata.len();

            if let Ok(accessed) = metadata.accessed() {
                if let Ok(elapsed) = now.duration_since(accessed) {
                    let days = elapsed.as_secs() / 86400;
                    if days >= 90 {
                        old_files_90d += 1;
                    }
                    if days >= 180 {
                        old_files_180d += 1;
                    }
                    if days >= 365 {
                        old_files_365d += 1;
                    }
                }
            }

            let file_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let extension = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            let last_accessed_days = metadata
                .accessed()
                .ok()
                .and_then(|a| now.duration_since(a).ok())
                .map(|e| e.as_secs() / 86400);

            all_files.push(TopFile {
                path: entry_path,
                name: file_name,
                size,
                last_accessed_days,
                extension,
            });
        }
    }

    all_files.sort_by_key(|f| std::cmp::Reverse(f.size));
    all_files.truncate(20);

    subfolder_sizes.sort_by_key(|s| std::cmp::Reverse(s.1));
    subfolder_sizes.truncate(10);

    Some(FolderSummary {
        name: name.to_string(),
        path: path.to_path_buf(),
        total_size,
        file_count,
        old_files_90d,
        old_files_180d,
        old_files_365d,
        top_files: all_files,
        subfolder_sizes,
    })
}

fn scan_files_recursive(
    dir: &Path,
    now: &SystemTime,
    file_count: &mut u64,
    old_90d: &mut u64,
    old_180d: &mut u64,
    old_365d: &mut u64,
    files: &mut Vec<TopFile>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() && !path.is_symlink() {
            let _ = scan_files_recursive(&path, now, file_count, old_90d, old_180d, old_365d, files);
        } else if metadata.is_file() {
            *file_count += 1;
            let size = metadata.len();

            if let Ok(accessed) = metadata.accessed() {
                if let Ok(elapsed) = now.duration_since(accessed) {
                    let days = elapsed.as_secs() / 86400;
                    if days >= 90 {
                        *old_90d += 1;
                    }
                    if days >= 180 {
                        *old_180d += 1;
                    }
                    if days >= 365 {
                        *old_365d += 1;
                    }
                }
            }

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            let last_accessed_days = metadata
                .accessed()
                .ok()
                .and_then(|a| now.duration_since(a).ok())
                .map(|e| e.as_secs() / 86400);

            files.push(TopFile {
                path,
                name: file_name,
                size,
                last_accessed_days,
                extension,
            });
        }
    }
    Ok(())
}
