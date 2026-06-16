use crate::advisor::models::{dir_size, human_bytes, Category, Recommendation, Risk};
use std::path::{Path, PathBuf};

pub fn scan_update_residue(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    if cfg!(target_os = "macos") {
        scan_shipit_caches(home, min_size, &mut recs);
        scan_sparkle_caches(home, min_size, &mut recs);
        scan_known_update_caches(home, min_size, &mut recs);
        scan_generic_update_dirs(home, min_size, &mut recs);
    }

    if cfg!(target_os = "linux") {
        scan_snap_old_revisions(min_size, &mut recs);
        scan_flatpak_unused_runtimes(home, min_size, &mut recs);
        scan_apt_deb_cache(min_size, &mut recs);
    }

    recs
}

fn scan_shipit_caches(home: &Path, min_size: u64, recs: &mut Vec<Recommendation>) {
    let caches_dir = home.join("Library/Caches");
    if !caches_dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(&caches_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let shipit_dir = path.join("ShipIt");
        if shipit_dir.exists() && shipit_dir.is_dir() {
            if let Ok(total) = dir_size(&shipit_dir) {
                if total >= min_size {
                    let app_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    recs.push(Recommendation {
                        category: Category::AutoUpdate,
                        path: shipit_dir.display().to_string(),
                        size: total,
                        risk: Risk::Safe,
                        reason: format!(
                            "ShipIt update cache for {}: {} (safe to remove, will re-download on next update)",
                            app_name,
                            human_bytes(total)
                        ),
                        suggested_command: format!("rm -rf '{}'", shipit_dir.display()),
                        last_accessed_days: get_last_accessed_days(&shipit_dir),
                    });
                }
            }
        }

        let shipit_state = path.join("ShipItState.plist");
        if shipit_state.exists() && shipit_state.is_file() {
            let meta = match std::fs::metadata(&shipit_state) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.len() >= min_size {
                let app_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                recs.push(Recommendation {
                    category: Category::AutoUpdate,
                    path: shipit_state.display().to_string(),
                    size: meta.len(),
                    risk: Risk::Safe,
                    reason: format!(
                        "ShipIt state for {}: {} (pending update state)",
                        app_name,
                        human_bytes(meta.len())
                    ),
                    suggested_command: format!("rm -f '{}'", shipit_state.display()),
                    last_accessed_days: get_last_accessed_days_file(&shipit_state),
                });
            }
        }
    }
}

fn scan_sparkle_caches(home: &Path, min_size: u64, recs: &mut Vec<Recommendation>) {
    let caches_dir = home.join("Library/Caches");
    if !caches_dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(&caches_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let sparkle_dir = path.join("Sparkle");
        if sparkle_dir.exists() && sparkle_dir.is_dir() {
            if let Ok(total) = dir_size(&sparkle_dir) {
                if total >= min_size {
                    let app_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    recs.push(Recommendation {
                        category: Category::AutoUpdate,
                        path: sparkle_dir.display().to_string(),
                        size: total,
                        risk: Risk::Safe,
                        reason: format!(
                            "Sparkle update cache for {}: {} (downloaded update files)",
                            app_name,
                            human_bytes(total)
                        ),
                        suggested_command: format!("rm -rf '{}'", sparkle_dir.display()),
                        last_accessed_days: get_last_accessed_days(&sparkle_dir),
                    });
                }
            }
        }
    }
}

fn scan_known_update_caches(home: &Path, min_size: u64, recs: &mut Vec<Recommendation>) {
    let known_paths: Vec<(PathBuf, &str, &str)> = vec![
        // VSCode
        (
            home.join("Library/Caches/com.microsoft.VSCode.ShipIt"),
            "VSCode ShipIt cache",
            "rm -rf ~/Library/Caches/com.microsoft.VSCode.ShipIt",
        ),
        (
            home.join("Library/Caches/com.microsoft.VSCode"),
            "VSCode update cache",
            "rm -rf ~/Library/Caches/com.microsoft.VSCode",
        ),
        // VSCode Insiders
        (
            home.join("Library/Caches/com.microsoft.VSCodeInsiders.ShipIt"),
            "VSCode Insiders ShipIt cache",
            "rm -rf ~/Library/Caches/com.microsoft.VSCodeInsiders.ShipIt",
        ),
        // Cursor
        (
            home.join("Library/Caches/com.todesktop.*"),
            "Cursor update cache",
            "rm -rf ~/Library/Caches/com.todesktop.*",
        ),
        (
            home.join(".cursor/updates"),
            "Cursor updates directory",
            "rm -rf ~/.cursor/updates",
        ),
        // Telegram
        (
            home.join("Library/Caches/com.llkd.Telegram"),
            "Telegram cache",
            "rm -rf ~/Library/Caches/com.llkd.Telegram",
        ),
        (
            home.join("Library/Containers/com.llkd.Telegram"),
            "Telegram container data",
            "rm -rf ~/Library/Containers/com.llkd.Telegram",
        ),
        // Slack
        (
            home.join("Library/Caches/com.slack.ShipIt"),
            "Slack ShipIt cache",
            "rm -rf ~/Library/Caches/com.slack.ShipIt",
        ),
        // Discord
        (
            home.join("Library/Caches/com.hnc.Discord"),
            "Discord cache",
            "rm -rf ~/Library/Caches/com.hnc.Discord",
        ),
        (
            home.join("Library/Caches/com.hnc.Discord.ShipIt"),
            "Discord ShipIt cache",
            "rm -rf ~/Library/Caches/com.hnc.Discord.ShipIt",
        ),
        // Chrome
        (
            home.join("Library/Caches/Google/Chrome"),
            "Chrome update cache",
            "rm -rf ~/Library/Caches/Google/Chrome",
        ),
        // Zoom
        (
            home.join("Library/Caches/us.zoom.xos"),
            "Zoom cache",
            "rm -rf ~/Library/Caches/us.zoom.xos",
        ),
        // Spotify
        (
            home.join("Library/Caches/com.spotify.client"),
            "Spotify cache",
            "rm -rf ~/Library/Caches/com.spotify.client",
        ),
        // Firefox
        (
            home.join("Library/Caches/Firefox"),
            "Firefox cache",
            "rm -rf ~/Library/Caches/Firefox",
        ),
    ];

    for (path, description, command) in &known_paths {
        // Handle glob-like patterns
        let path_str = path.display().to_string();
        if path_str.contains('*') {
            scan_glob_pattern(&path_str, description, command, min_size, recs);
            continue;
        }

        if !path.exists() {
            continue;
        }

        if path.is_dir() {
            if let Ok(total) = dir_size(path) {
                if total >= min_size {
                    recs.push(Recommendation {
                        category: Category::AutoUpdate,
                        path: path.display().to_string(),
                        size: total,
                        risk: Risk::Low,
                        reason: format!("{}: {}", description, human_bytes(total)),
                        suggested_command: command.to_string(),
                        last_accessed_days: get_last_accessed_days(path),
                    });
                }
            }
        } else if path.is_file() {
            if let Ok(meta) = std::fs::metadata(path) {
                if meta.len() >= min_size {
                    recs.push(Recommendation {
                        category: Category::AutoUpdate,
                        path: path.display().to_string(),
                        size: meta.len(),
                        risk: Risk::Low,
                        reason: format!("{}: {}", description, human_bytes(meta.len())),
                        suggested_command: format!("rm -f '{}'", path.display()),
                        last_accessed_days: get_last_accessed_days_file(path),
                    });
                }
            }
        }
    }
}

fn scan_glob_pattern(pattern: &str, description: &str, _command: &str, min_size: u64, recs: &mut Vec<Recommendation>) {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() != 2 {
        return;
    }

    let prefix = parts[0];
    let suffix = parts[1];
    let parent = Path::new(prefix);

    if !parent.exists() {
        return;
    }

    let parent_dir = match parent.parent() {
        Some(p) => p,
        None => return,
    };

    let dir_name_prefix = parent
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let entries = match std::fs::read_dir(parent_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with(dir_name_prefix) && name.ends_with(suffix) {
            if let Ok(total) = dir_size(&path) {
                if total >= min_size {
                    let app_name = name.to_string();
                    recs.push(Recommendation {
                        category: Category::AutoUpdate,
                        path: path.display().to_string(),
                        size: total,
                        risk: Risk::Low,
                        reason: format!("{} ({}): {}", description, app_name, human_bytes(total)),
                        suggested_command: format!("rm -rf '{}'", path.display()),
                        last_accessed_days: get_last_accessed_days(&path),
                    });
                }
            }
        }
    }
}

fn scan_generic_update_dirs(home: &Path, min_size: u64, recs: &mut Vec<Recommendation>) {
    let app_support = home.join("Library/Application Support");
    if !app_support.exists() {
        return;
    }

    let entries = match std::fs::read_dir(&app_support) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let update_dirs = ["updates", "pending-update", "Updates"];
        for update_dir_name in &update_dirs {
            let update_dir = path.join(update_dir_name);
            if update_dir.exists() && update_dir.is_dir() {
                if let Ok(total) = dir_size(&update_dir) {
                    if total >= min_size {
                        let app_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        recs.push(Recommendation {
                            category: Category::AutoUpdate,
                            path: update_dir.display().to_string(),
                            size: total,
                            risk: Risk::Low,
                            reason: format!(
                                "Update files for {}: {} (downloaded but possibly stale updates)",
                                app_name,
                                human_bytes(total)
                            ),
                            suggested_command: format!("rm -rf '{}'", update_dir.display()),
                            last_accessed_days: get_last_accessed_days(&update_dir),
                        });
                    }
                }
            }
        }
    }
}

fn scan_snap_old_revisions(min_size: u64, recs: &mut Vec<Recommendation>) {
    use std::collections::HashMap;

    let snap_dir = Path::new("/var/lib/snapd/snaps");
    if !snap_dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(snap_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    // Group snap files by package name, keep only old revisions
    let mut by_name: HashMap<String, Vec<(PathBuf, u64)>> = HashMap::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("snap") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        // snap filenames look like "firefox_1234.snap" — group by base before last _
        let base = name
            .rsplit_once('_')
            .map(|(n, _)| n.to_string())
            .unwrap_or(name);

        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        by_name.entry(base).or_default().push((path, size));
    }

    for (base, revisions) in &by_name {
        if revisions.len() < 2 {
            continue;
        }
        // All revisions except the latest are old
        let mut sorted = revisions.clone();
        sorted.sort_by_key(|(p, _)| p.clone());
        let old_revisions = &sorted[..sorted.len() - 1];
        let old_total: u64 = old_revisions.iter().map(|(_, s)| *s).sum();

        if old_total >= min_size {
            let old_paths: Vec<String> = old_revisions
                .iter()
                .map(|(p, _)| format!("'{}'", p.display()))
                .collect();
            recs.push(Recommendation {
                category: Category::AutoUpdate,
                path: snap_dir.display().to_string(),
                size: old_total,
                risk: Risk::Safe,
                reason: format!(
                    "{} old snap revision(s) for '{}': {} (safe to remove, current revision is kept)",
                    old_revisions.len(),
                    base,
                    human_bytes(old_total)
                ),
                suggested_command: format!("sudo rm -f {}", old_paths.join(" ")),
                last_accessed_days: None,
            });
        }
    }
}

fn scan_flatpak_unused_runtimes(home: &Path, min_size: u64, recs: &mut Vec<Recommendation>) {
    let runtime_dirs = [
        PathBuf::from("/var/lib/flatpak/runtime"),
        home.join(".local/share/flatpak/runtime"),
    ];

    for runtime_dir in &runtime_dirs {
        if !runtime_dir.exists() {
            continue;
        }
        if let Ok(total) = dir_size(runtime_dir) {
            if total >= min_size {
                recs.push(Recommendation {
                    category: Category::AutoUpdate,
                    path: runtime_dir.display().to_string(),
                    size: total,
                    risk: Risk::Low,
                    reason: format!(
                        "Flatpak runtime data: {} (may include unused runtimes)",
                        human_bytes(total)
                    ),
                    suggested_command: "flatpak uninstall --unused".to_string(),
                    last_accessed_days: get_last_accessed_days(runtime_dir),
                });
            }
        }
    }
}

fn scan_apt_deb_cache(min_size: u64, recs: &mut Vec<Recommendation>) {
    let apt_cache = Path::new("/var/cache/apt/archives");
    if !apt_cache.exists() {
        return;
    }
    if let Ok(total) = dir_size(apt_cache) {
        if total >= min_size {
            recs.push(Recommendation {
                category: Category::AutoUpdate,
                path: apt_cache.display().to_string(),
                size: total,
                risk: Risk::Safe,
                reason: format!(
                    "APT downloaded package cache: {} (already installed, safe to clean)",
                    human_bytes(total)
                ),
                suggested_command: "sudo apt-get clean".to_string(),
                last_accessed_days: None,
            });
        }
    }
}

fn get_last_accessed_days(path: &Path) -> Option<u64> {
    let metadata = std::fs::metadata(path).ok()?;
    let accessed = metadata.accessed().ok()?;
    let elapsed = accessed.elapsed().ok()?;
    Some(elapsed.as_secs() / 86400)
}

fn get_last_accessed_days_file(path: &Path) -> Option<u64> {
    let metadata = std::fs::metadata(path).ok()?;
    let accessed = metadata.accessed().ok()?;
    let elapsed = accessed.elapsed().ok()?;
    Some(elapsed.as_secs() / 86400)
}
