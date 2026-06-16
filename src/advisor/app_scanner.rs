use crate::advisor::models::{dir_size, AppInfo};
use std::path::{Path, PathBuf};

pub fn scan_applications() -> Vec<AppInfo> {
    if cfg!(target_os = "linux") {
        scan_linux_apps()
    } else {
        scan_macos_apps()
    }
}

fn scan_macos_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();

    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => return apps,
    };

    let app_dirs = [
        PathBuf::from("/Applications"),
        home.join("Applications"),
    ];

    for app_dir in &app_dirs {
        if !app_dir.exists() {
            continue;
        }
        scan_app_directory(app_dir, &home, &mut apps);
    }

    apps.sort_by_key(|b| std::cmp::Reverse(b.total_size));
    apps
}

fn scan_app_directory(dir: &Path, home: &Path, apps: &mut Vec<AppInfo>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "app" {
            continue;
        }

        if let Some(app_info) = scan_single_app(&path, home) {
            apps.push(app_info);
        }
    }
}

fn scan_single_app(app_path: &Path, home: &Path) -> Option<AppInfo> {
    let name = app_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let bundle_path = app_path.display().to_string();

    let info_plist = app_path.join("Contents/Info.plist");
    let bundle_id = read_bundle_id(&info_plist).unwrap_or_default();
    let version = read_app_version(&info_plist);

    let app_size = dir_size(app_path).unwrap_or(0);

    let mut support_dirs = Vec::new();
    let mut support_size = 0u64;
    let mut cache_size = 0u64;

    let library = home.join("Library");

    let candidate_dirs = if !bundle_id.is_empty() {
        vec![
            library.join("Application Support").join(&bundle_id),
            library.join("Application Support").join(&name),
            library.join("Caches").join(&bundle_id),
            library.join("Caches").join(&name),
            library.join("Containers").join(&bundle_id),
            library.join("Preferences").join(format!("{}.plist", bundle_id)),
            library.join("Logs").join(&bundle_id),
            library.join("Logs").join(&name),
            library.join("Saved Application State").join(format!("{}.savedState", bundle_id)),
            library.join("HTTPStorages").join(&bundle_id),
            library.join("WebKit").join(&bundle_id),
        ]
    } else {
        vec![
            library.join("Application Support").join(&name),
            library.join("Caches").join(&name),
            library.join("Logs").join(&name),
        ]
    };

    for candidate in &candidate_dirs {
        if candidate.exists() {
            let size = if candidate.is_dir() {
                dir_size(candidate).unwrap_or(0)
            } else if candidate.is_file() {
                std::fs::metadata(candidate).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };

            if size > 0 {
                support_dirs.push(candidate.display().to_string());

                let path_str = candidate.display().to_string();
                if path_str.contains("/Caches/") {
                    cache_size += size;
                } else {
                    support_size += size;
                }
            }
        }
    }

    let total_size = app_size + support_size + cache_size;
    let last_used_days = get_last_used_days(app_path);

    Some(AppInfo {
        name,
        bundle_path,
        bundle_id,
        total_size,
        app_size,
        support_size,
        cache_size,
        last_used_days,
        version,
        support_dirs,
    })
}

fn read_bundle_id(plist_path: &Path) -> Option<String> {
    if !plist_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(plist_path).ok()?;
    extract_plist_string_value(&content, "CFBundleIdentifier")
}

fn read_app_version(plist_path: &Path) -> Option<String> {
    if !plist_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(plist_path).ok()?;
    extract_plist_string_value(&content, "CFBundleShortVersionString")
        .or_else(|| extract_plist_string_value(&content, "CFBundleVersion"))
}

fn extract_plist_string_value(plist_content: &str, key: &str) -> Option<String> {
    let key_tag = format!("<key>{}</key>", key);
    let key_pos = plist_content.find(&key_tag)?;
    let after_key = &plist_content[key_pos + key_tag.len()..];

    let start = after_key.find("<string>")? + 8;
    let end = after_key[start..].find("</string>")?;
    Some(after_key[start..start + end].to_string())
}

fn get_last_used_days(app_path: &Path) -> Option<u64> {
    // metadata.accessed() is unreliable on APFS — use a multi-signal approach:
    // 1. Check if process is currently running
    // 2. Check mtime of bundle, Caches dir, Preferences, Saved Application State
    // 3. Take the most recent signal

    let name = app_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check if currently running
    if is_process_running(&name) {
        return Some(0);
    }

    let mut most_recent_secs: Option<u64> = None;

    // Check bundle mtime
    if let Ok(meta) = std::fs::metadata(app_path) {
        if let Ok(modified) = meta.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                let days = elapsed.as_secs() / 86400;
                most_recent_secs = Some(
                    most_recent_secs.map_or(days, |prev: u64| prev.min(days)),
                );
            }
        }
    }

    // Check Saved Application State (most reliable for "last launched")
    if let Ok(home) = std::env::var("HOME") {
        let info_plist = app_path.join("Contents/Info.plist");
        let bundle_id = read_bundle_id(&info_plist).unwrap_or_default();

        if !bundle_id.is_empty() {
            let saved_state = PathBuf::from(&home)
                .join("Library/Saved Application State")
                .join(format!("{}.savedState", bundle_id));
            if let Ok(meta) = std::fs::metadata(&saved_state) {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        let days = elapsed.as_secs() / 86400;
                        most_recent_secs = Some(
                            most_recent_secs.map_or(days, |prev: u64| prev.min(days)),
                        );
                    }
                }
            }

            // Check Preferences plist mtime
            let prefs = PathBuf::from(&home)
                .join("Library/Preferences")
                .join(format!("{}.plist", bundle_id));
            if let Ok(meta) = std::fs::metadata(&prefs) {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        let days = elapsed.as_secs() / 86400;
                        most_recent_secs = Some(
                            most_recent_secs.map_or(days, |prev: u64| prev.min(days)),
                        );
                    }
                }
            }
        }
    }

    most_recent_secs
}

fn is_process_running(name: &str) -> bool {
    // Check /proc on Linux, pgrep on macOS
    if cfg!(target_os = "macos") {
        if let Ok(output) = std::process::Command::new("pgrep")
            .arg("-x")
            .arg(name)
            .output()
        {
            return output.status.success();
        }
        // Also check case-insensitive partial match
        if let Ok(output) = std::process::Command::new("pgrep")
            .arg("-i")
            .arg(name)
            .output()
        {
            return output.status.success();
        }
    } else if cfg!(target_os = "linux") {
        if let Ok(output) = std::process::Command::new("pgrep")
            .arg("-x")
            .arg(name)
            .output()
        {
            return output.status.success();
        }
    }
    false
}

fn scan_linux_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();

    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => return apps,
    };

    scan_dpkg_packages(&mut apps);
    scan_snap_packages(&mut apps);
    scan_flatpak_apps(&home, &mut apps);

    apps.sort_by_key(|b| std::cmp::Reverse(b.total_size));
    apps
}

fn scan_dpkg_packages(apps: &mut Vec<AppInfo>) {
    let status_file = Path::new("/var/lib/dpkg/status");
    if !status_file.exists() {
        return;
    }

    let content = match std::fs::read_to_string(status_file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut current_name = String::new();
    let mut current_version = String::new();
    let mut current_size: u64 = 0;
    let mut current_status = String::new();

    for line in content.lines() {
        if line.is_empty() {
            if !current_name.is_empty()
                && current_size > 0
                && current_status.contains("installed")
                && !current_status.contains("config-files")
            {
                apps.push(AppInfo {
                    name: current_name.clone(),
                    bundle_path: format!("/usr/{}", current_name),
                    bundle_id: current_name.clone(),
                    total_size: current_size,
                    app_size: current_size,
                    support_size: 0,
                    cache_size: 0,
                    last_used_days: None,
                    version: if current_version.is_empty() {
                        None
                    } else {
                        Some(current_version.clone())
                    },
                    support_dirs: Vec::new(),
                });
            }
            current_name.clear();
            current_version.clear();
            current_size = 0;
            current_status.clear();
            continue;
        }

        if let Some(name) = line.strip_prefix("Package: ") {
            current_name = name.trim().to_string();
        } else if let Some(ver) = line.strip_prefix("Version: ") {
            current_version = ver.trim().to_string();
        } else if let Some(size_str) = line.strip_prefix("Installed-Size: ") {
            current_size = size_str.trim().parse::<u64>().unwrap_or(0) * 1024;
        } else if let Some(status) = line.strip_prefix("Status: ") {
            current_status = status.trim().to_string();
        }
    }
}

fn scan_snap_packages(apps: &mut Vec<AppInfo>) {
    let snap_dir = Path::new("/var/lib/snapd/snaps");
    if !snap_dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(snap_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "snap" {
                if let Ok(meta) = std::fs::metadata(&path) {
                    let name = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let revision = name
                        .rsplit('_')
                        .next()
                        .unwrap_or("")
                        .to_string();
                    let base_name = name
                        .rsplit_once('_')
                        .map(|(n, _)| n.to_string())
                        .unwrap_or_else(|| name.clone());

                    apps.push(AppInfo {
                        name: format!("snap:{}", base_name),
                        bundle_path: path.display().to_string(),
                        bundle_id: format!("snap.{}", base_name),
                        total_size: meta.len(),
                        app_size: meta.len(),
                        support_size: 0,
                        cache_size: 0,
                        last_used_days: None,
                        version: Some(revision),
                        support_dirs: Vec::new(),
                    });
                }
            }
        }
    }
}

fn scan_flatpak_apps(home: &Path, apps: &mut Vec<AppInfo>) {
    let flatpak_dirs = [
        PathBuf::from("/var/lib/flatpak/app"),
        home.join(".local/share/flatpak/app"),
    ];

    for flatpak_dir in &flatpak_dirs {
        if !flatpak_dir.exists() {
            continue;
        }

        let entries = match std::fs::read_dir(flatpak_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let size = dir_size(&path).unwrap_or(0);
            if size > 0 {
                apps.push(AppInfo {
                    name: format!("flatpak:{}", name),
                    bundle_path: path.display().to_string(),
                    bundle_id: name.clone(),
                    total_size: size,
                    app_size: size,
                    support_size: 0,
                    cache_size: 0,
                    last_used_days: None,
                    version: None,
                    support_dirs: Vec::new(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_plist_string_value() {
        let plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="0.9">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.example.app</string>
    <key>CFBundleShortVersionString</key>
    <string>1.2.3</string>
</dict>
</plist>"#;

        assert_eq!(
            extract_plist_string_value(plist, "CFBundleIdentifier"),
            Some("com.example.app".to_string())
        );
        assert_eq!(
            extract_plist_string_value(plist, "CFBundleShortVersionString"),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            extract_plist_string_value(plist, "NonExistent"),
            None
        );
    }
}
