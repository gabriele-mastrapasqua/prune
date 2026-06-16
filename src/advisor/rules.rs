use crate::advisor::models::*;
use crate::advisor::paths::PlatformPaths;

pub fn evaluate_rules(base: &std::path::Path, no_dev: bool, _no_ai: bool) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();
    let paths = PlatformPaths::new();

    if !no_dev {
        // Dead simulator containers (macOS only)
        if cfg!(target_os = "macos") {
            scan_dead_simulators(base, &mut recommendations);
        }

        // Xcode paths (macOS only)
        if cfg!(target_os = "macos") {
            scan_xcode_derived_data(base, &mut recommendations);
            scan_xcode_device_support(base, &mut recommendations);
            scan_xcode_archives(base, &mut recommendations);
        }

        // Docker Desktop (cross-platform)
        scan_docker(&paths, &mut recommendations);
    }

    // Old installers in Downloads (cross-platform)
    scan_downloads_installers(base, &mut recommendations);

    // macOS installer apps in /Applications (macOS only)
    if cfg!(target_os = "macos") {
        scan_macos_installers(&mut recommendations);
    }

    // Trash (cross-platform)
    scan_trash(base, &mut recommendations);

    // Linux-specific rules
    if cfg!(target_os = "linux") {
        scan_linux_rotated_logs(&mut recommendations);
        scan_pacman_cache(&mut recommendations);
        scan_dnf_cache(&mut recommendations);
    }

    recommendations
}

fn scan_dead_simulators(base: &std::path::Path, recs: &mut Vec<Recommendation>) {
    let sim_path = base.join("Library/Developer/CoreSimulator/Devices");
    if !sim_path.exists() {
        return;
    }
    let entries = match std::fs::read_dir(&sim_path) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let dead_path = entry
            .path()
            .join("data/Library/Caches/com.apple.containermanagerd/Dead");
        if dead_path.exists() && dead_path.is_dir() {
            if let Ok(total) = dir_size(&dead_path) {
                if total > 50 * 1024 * 1024 {
                    recs.push(Recommendation {
                        category: Category::Dev(DevKind::Simulator),
                        path: dead_path.display().to_string(),
                        size: total,
                        risk: Risk::Safe,
                        reason: format!("Dead simulator containers: {}", human_bytes(total)),
                        suggested_command: "xcrun simctl delete unavailable".to_string(),
                        last_accessed_days: None,
                    });
                }
            }
        }
    }
}

fn scan_xcode_derived_data(base: &std::path::Path, recs: &mut Vec<Recommendation>) {
    let dd = base.join("Library/Developer/Xcode/DerivedData");
    if !dd.exists() {
        return;
    }
    if let Ok(total) = dir_size(&dd) {
        if total > 500 * 1024 * 1024 {
            recs.push(Recommendation {
                category: Category::Dev(DevKind::Xcode),
                path: dd.display().to_string(),
                size: total,
                risk: Risk::Safe,
                reason: format!("Xcode build cache: {}", human_bytes(total)),
                suggested_command: "rm -rf ~/Library/Developer/Xcode/DerivedData/*".to_string(),
                last_accessed_days: None,
            });
        }
    }
}

fn scan_xcode_device_support(base: &std::path::Path, recs: &mut Vec<Recommendation>) {
    let ds = base.join("Library/Developer/Xcode/iOS DeviceSupport");
    if !ds.exists() {
        return;
    }
    if let Ok(total) = dir_size(&ds) {
        if total > 2 * 1024 * 1024 * 1024 {
            recs.push(Recommendation {
                category: Category::Dev(DevKind::Xcode),
                path: ds.display().to_string(),
                size: total,
                risk: Risk::Safe,
                reason: format!("iOS DeviceSupport: {}", human_bytes(total)),
                suggested_command: "rm -rf ~/Library/Developer/Xcode/iOS\\ DeviceSupport/*"
                    .to_string(),
                last_accessed_days: None,
            });
        }
    }
}

fn scan_xcode_archives(base: &std::path::Path, recs: &mut Vec<Recommendation>) {
    let archives = base.join("Library/Developer/Xcode/Archives");
    if !archives.exists() {
        return;
    }
    if let Ok(total) = dir_size(&archives) {
        if total > 1024 * 1024 * 1024 {
            recs.push(Recommendation {
                category: Category::Dev(DevKind::Xcode),
                path: archives.display().to_string(),
                size: total,
                risk: Risk::Low,
                reason: format!("Xcode Archives: {}", human_bytes(total)),
                suggested_command: "rm -rf ~/Library/Developer/Xcode/Archives/*".to_string(),
                last_accessed_days: None,
            });
        }
    }
}

fn scan_docker(paths: &PlatformPaths, recs: &mut Vec<Recommendation>) {
    // Use platform-specific Docker path
    if let Some(docker_path) = paths.docker_data() {
        if docker_path.exists() {
            if docker_path.is_file() {
                // Docker.raw file
                if let Ok(meta) = std::fs::metadata(&docker_path) {
                    if meta.len() > 5 * 1024 * 1024 * 1024 {
                        recs.push(Recommendation {
                            category: Category::Dev(DevKind::Docker),
                            path: docker_path.display().to_string(),
                            size: meta.len(),
                            risk: Risk::Medium,
                            reason: format!(
                                "Docker Desktop disk image: {}",
                                human_bytes(meta.len())
                            ),
                            suggested_command: "docker system prune -af --volumes".to_string(),
                            last_accessed_days: None,
                        });
                    }
                }
            } else if docker_path.is_dir() {
                // Docker data directory
                if let Ok(total) = dir_size(&docker_path) {
                    if total > 5 * 1024 * 1024 * 1024 {
                        recs.push(Recommendation {
                            category: Category::Dev(DevKind::Docker),
                            path: docker_path.display().to_string(),
                            size: total,
                            risk: Risk::Medium,
                            reason: format!("Docker Desktop data: {}", human_bytes(total)),
                            suggested_command: "docker system prune -af".to_string(),
                            last_accessed_days: None,
                        });
                    }
                }
            }
        }
    }
}

fn scan_downloads_installers(base: &std::path::Path, recs: &mut Vec<Recommendation>) {
    let downloads = base.join("Downloads");
    if !downloads.exists() {
        return;
    }
    let entries = match std::fs::read_dir(&downloads) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut total_installer_size: u64 = 0;
    let mut installer_count: u32 = 0;
    let mut largest_file: Option<(String, u64)> = None;

    // Cross-platform installer extensions
    let installer_extensions = if cfg!(target_os = "macos") {
        vec!["dmg", "pkg", "iso", "xip", "zip"]
    } else if cfg!(target_os = "windows") {
        vec!["exe", "msi", "iso", "zip"]
    } else {
        // Linux
        vec!["deb", "rpm", "AppImage", "iso", "tar.gz", "zip"]
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                let is_installer = match path.extension().and_then(|e| e.to_str()) {
                    Some(ext) => installer_extensions.contains(&ext),
                    _ => false,
                };
                if is_installer && metadata.len() > 100 * 1024 * 1024 {
                    total_installer_size += metadata.len();
                    installer_count += 1;
                    match &largest_file {
                        Some((_, prev_size)) if metadata.len() > *prev_size => {
                            largest_file = Some((path.display().to_string(), metadata.len()));
                        }
                        None => {
                            largest_file = Some((path.display().to_string(), metadata.len()));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    if installer_count > 0 && total_installer_size > 500 * 1024 * 1024 {
        let largest_info = match &largest_file {
            Some((path, size)) => format!("\n       Largest: {} ({})", path, human_bytes(*size)),
            None => String::new(),
        };

        let suggested_cmd = if cfg!(target_os = "windows") {
            "dir /O-S \"%USERPROFILE%\\Downloads\\*.exe\" \"%USERPROFILE%\\Downloads\\*.msi\""
                .to_string()
        } else {
            format!(
                "ls -lhS ~/Downloads/*.{{{}}} 2>/dev/null",
                installer_extensions.join(",")
            )
        };

        recs.push(Recommendation {
            category: Category::Installer,
            path: downloads.display().to_string(),
            size: total_installer_size,
            risk: Risk::Review,
            reason: format!(
                "{} installer files ({}) in Downloads{}",
                installer_count,
                human_bytes(total_installer_size),
                largest_info
            ),
            suggested_command: suggested_cmd,
            last_accessed_days: None,
        });
    }
}

fn scan_macos_installers(recs: &mut Vec<Recommendation>) {
    let apps = std::path::Path::new("/Applications");
    if !apps.exists() {
        return;
    }
    let entries = match std::fs::read_dir(apps) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("Install macOS") && name_str.ends_with(".app") {
            let path = entry.path();
            if let Ok(total) = dir_size(&path) {
                if total > 1024 * 1024 * 1024 {
                    recs.push(Recommendation {
                        category: Category::Installer,
                        path: path.display().to_string(),
                        size: total,
                        risk: Risk::Safe,
                        reason: format!("macOS installer app: {}", human_bytes(total)),
                        suggested_command: format!("rm -rf '{}'", path.display()),
                        last_accessed_days: None,
                    });
                }
            }
        }
    }
}

fn scan_linux_rotated_logs(recs: &mut Vec<Recommendation>) {
    let var_log = std::path::Path::new("/var/log");
    if !var_log.exists() {
        return;
    }
    let entries = match std::fs::read_dir(var_log) {
        Ok(e) => e,
        Err(_) => return,
    };
    let mut total_rotated: u64 = 0;
    let mut count: u32 = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if name.ends_with(".gz") || name.ends_with(".old") || name.ends_with(".xz") || name.ends_with(".bz2") {
            if let Ok(meta) = std::fs::metadata(&path) {
                total_rotated += meta.len();
                count += 1;
            }
        }
    }
    if count > 0 && total_rotated > 100 * 1024 * 1024 {
        recs.push(Recommendation {
            category: Category::Log,
            path: var_log.display().to_string(),
            size: total_rotated,
            risk: Risk::Safe,
            reason: format!(
                "{} rotated log files in /var/log ({})",
                count,
                human_bytes(total_rotated)
            ),
            suggested_command: "sudo find /var/log -name '*.gz' -o -name '*.old' -o -name '*.xz' -o -name '*.bz2' | xargs sudo rm -f".to_string(),
            last_accessed_days: None,
        });
    }
}

fn scan_pacman_cache(recs: &mut Vec<Recommendation>) {
    let cache = std::path::Path::new("/var/cache/pacman/pkg");
    if !cache.exists() {
        return;
    }
    if let Ok(total) = dir_size(cache) {
        if total > 500 * 1024 * 1024 {
            recs.push(Recommendation {
                category: Category::Cache,
                path: cache.display().to_string(),
                size: total,
                risk: Risk::Safe,
                reason: format!("Pacman package cache: {}", human_bytes(total)),
                suggested_command: "sudo pacman -Sc".to_string(),
                last_accessed_days: None,
            });
        }
    }
}

fn scan_dnf_cache(recs: &mut Vec<Recommendation>) {
    let cache = std::path::Path::new("/var/cache/dnf");
    if !cache.exists() {
        return;
    }
    if let Ok(total) = dir_size(cache) {
        if total > 200 * 1024 * 1024 {
            recs.push(Recommendation {
                category: Category::Cache,
                path: cache.display().to_string(),
                size: total,
                risk: Risk::Safe,
                reason: format!("DNF package cache: {}", human_bytes(total)),
                suggested_command: "sudo dnf clean all".to_string(),
                last_accessed_days: None,
            });
        }
    }
}

fn scan_trash(base: &std::path::Path, recs: &mut Vec<Recommendation>) {
    // Cross-platform trash detection
    let trash = if cfg!(target_os = "macos") {
        base.join(".Trash")
    } else if cfg!(target_os = "windows") {
        // Windows Recycle Bin is handled differently, skip for now
        return;
    } else {
        // Linux: ~/.local/share/Trash
        base.join(".local/share/Trash")
    };

    if !trash.exists() {
        return;
    }
    if let Ok(total) = dir_size(&trash) {
        if total > 100 * 1024 * 1024 {
            let cmd = if cfg!(target_os = "macos") {
                "rm -rf ~/.Trash/*".to_string()
            } else {
                "rm -rf ~/.local/share/Trash/files/*".to_string()
            };

            recs.push(Recommendation {
                category: Category::SystemTemp,
                path: trash.display().to_string(),
                size: total,
                risk: Risk::Safe,
                reason: format!("Trash: {}", human_bytes(total)),
                suggested_command: cmd,
                last_accessed_days: None,
            });
        }
    }
}
