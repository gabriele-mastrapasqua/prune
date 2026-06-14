use crate::advisor::models::*;
use std::path::{Path, PathBuf};

pub fn scan_all_versions(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // Node version managers
    recs.extend(scan_nvm(home, min_size));
    recs.extend(scan_fnm(home, min_size));
    recs.extend(scan_volta(home, min_size));

    // Python version managers
    recs.extend(scan_pyenv(home, min_size));
    recs.extend(scan_conda(home, min_size));

    // Rust
    recs.extend(scan_rustup(home, min_size));

    recs
}

// ── Node Version Managers ──────────────────────────────

fn scan_nvm(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let nvm_dir = home.join(".nvm/versions/node");
    if !nvm_dir.exists() {
        return vec![];
    }

    // Collect all versions first to resolve the "current" one
    let mut versions: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let version = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                versions.push((version, path));
            }
        }
    }

    let default_alias = get_nvm_default_version(home);
    let current_version = resolve_nvm_current_version(&default_alias, &versions);

    let mut recs = Vec::new();

    for (version, path) in &versions {
        let is_current = current_version.as_ref().map_or(false, |cv| version == cv);

        if let Ok(total) = dir_size(path) {
            if total < min_size {
                continue;
            }

            let major = extract_major_version(version);
            let is_deprecated = is_node_deprecated(major);

            let (risk, reason) = if is_current {
                (Risk::Low, format!("Node.js {} (current default)", version))
            } else if is_deprecated {
                (Risk::Safe, format!("Node.js {} is EOL (End of Life)", version))
            } else {
                (Risk::Low, format!("Node.js {} (not current)", version))
            };

            let cmd = if is_current {
                format!("# Keep this version (current default): {}", version)
            } else {
                format!("nvm uninstall {}", version.trim_start_matches('v'))
            };

            recs.push(Recommendation {
                category: Category::Dev(DevKind::VersionManager),
                path: path.display().to_string(),
                size: total,
                risk,
                reason,
                suggested_command: cmd,
                last_accessed_days: get_last_accessed_days(path),
            });
        }
    }

    recs
}

fn resolve_nvm_current_version(alias: &Option<String>, versions: &[(String, std::path::PathBuf)]) -> Option<String> {
    let alias = alias.as_ref()?;

    // If alias is a full version like "v20.20.2" or "20.20.2"
    if alias.contains('.') {
        let normalized = if alias.starts_with('v') {
            alias.clone()
        } else {
            format!("v{}", alias)
        };
        if versions.iter().any(|(v, _)| v == &normalized) {
            return Some(normalized);
        }
    }

    // If alias is a major version like "20" or "lts/*", find the highest matching version
    let major_str = alias.trim_start_matches("lts/");
    if let Ok(major) = major_str.parse::<u32>() {
        let mut matching: Vec<&str> = versions
            .iter()
            .filter(|(v, _)| {
                let cleaned = v.trim_start_matches('v');
                cleaned.split('.').next().and_then(|s| s.parse::<u32>().ok()) == Some(major)
            })
            .map(|(v, _)| v.as_str())
            .collect();

        // Sort by semver descending to get the highest version
        matching.sort_by(|a, b| {
            let a_parts = parse_semver(a);
            let b_parts = parse_semver(b);
            b_parts.cmp(&a_parts)
        });

        return matching.first().map(|s| s.to_string());
    }

    None
}

fn parse_semver(version: &str) -> (u32, u32, u32) {
    let cleaned = version.trim_start_matches('v');
    let parts: Vec<u32> = cleaned
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

fn get_nvm_default_version(home: &Path) -> Option<String> {
    let alias_file = home.join(".nvm/alias/default");
    if alias_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&alias_file) {
            return Some(content.trim().to_string());
        }
    }
    None
}

fn scan_fnm(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let fnm_dir = home.join(".fnm/node-versions");
    if !fnm_dir.exists() {
        return vec![];
    }

    let current_version = get_fnm_default_version(home);
    let mut recs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&fnm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let version = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let is_current = current_version.as_ref().map_or(false, |cv| {
                version.contains(cv)
            });

            let install_dir = path.join("installation");
            let scan_path = if install_dir.exists() { &install_dir } else { &path };

            if let Ok(total) = dir_size(scan_path) {
                if total < min_size {
                    continue;
                }

                let major = extract_major_version(&version);
                let is_deprecated = is_node_deprecated(major);

                let (risk, reason) = if is_current {
                    (Risk::Low, format!("Node.js {} (fnm current)", version))
                } else if is_deprecated {
                    (Risk::Safe, format!("Node.js {} is EOL (fnm)", version))
                } else {
                    (Risk::Low, format!("Node.js {} (fnm, not current)", version))
                };

                let cmd = if is_current {
                    format!("# Keep this version (fnm current): {}", version)
                } else {
                    format!("fnm uninstall {}", version.trim_start_matches('v'))
                };

                recs.push(Recommendation {
                    category: Category::Dev(DevKind::VersionManager),
                    path: path.display().to_string(),
                    size: total,
                    risk,
                    reason,
                    suggested_command: cmd,
                    last_accessed_days: get_last_accessed_days(scan_path),
                });
            }
        }
    }

    recs
}

fn get_fnm_default_version(home: &Path) -> Option<String> {
    let alias_file = home.join(".fnm/aliases/default");
    if alias_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&alias_file) {
            return Some(content.trim().to_string());
        }
    }
    let default_file = home.join(".fnm/.default");
    if default_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&default_file) {
            return Some(content.trim().to_string());
        }
    }
    None
}

fn scan_volta(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let volta_node_dir = home.join(".volta/tools/image/node");
    if !volta_node_dir.exists() {
        return vec![];
    }

    let mut recs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&volta_node_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let version = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if let Ok(total) = dir_size(&path) {
                if total < min_size {
                    continue;
                }

                let major = extract_major_version(&version);
                let is_deprecated = is_node_deprecated(major);

                let (risk, reason) = if is_deprecated {
                    (Risk::Safe, format!("Node.js {} is EOL (Volta)", version))
                } else {
                    (Risk::Low, format!("Node.js {} (Volta)", version))
                };

                recs.push(Recommendation {
                    category: Category::Dev(DevKind::VersionManager),
                    path: path.display().to_string(),
                    size: total,
                    risk,
                    reason,
                    suggested_command: format!("volta uninstall node@{}", version),
                    last_accessed_days: get_last_accessed_days(&path),
                });
            }
        }
    }

    recs
}

// ── Python Version Managers ────────────────────────────

fn scan_pyenv(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let pyenv_dir = home.join(".pyenv/versions");
    if !pyenv_dir.exists() {
        return vec![];
    }

    let current_version = get_pyenv_default_version(home);
    let mut recs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&pyenv_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let version = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let is_current = current_version.as_ref().map_or(false, |cv| {
                version == *cv || version.starts_with(&format!("{}.", cv))
            });

            if let Ok(total) = dir_size(&path) {
                if total < min_size {
                    continue;
                }

                // Check for heavy ML libraries
                let ml_info = check_python_ml_libs(&path);

                let (risk, reason) = if is_current {
                    if ml_info.is_empty() {
                        (Risk::Low, format!("Python {} (pyenv current)", version))
                    } else {
                        (Risk::Medium, format!("Python {} (current) with ML libs: {}", version, ml_info))
                    }
                } else if !ml_info.is_empty() {
                    (Risk::Low, format!("Python {} (not current) with ML libs: {}", version, ml_info))
                } else {
                    (Risk::Safe, format!("Python {} (not current, no ML libs)", version))
                };

                let cmd = if is_current {
                    format!("# Keep this version (pyenv current): {}", version)
                } else {
                    format!("pyenv uninstall {}", version)
                };

                recs.push(Recommendation {
                    category: Category::Dev(DevKind::VersionManager),
                    path: path.display().to_string(),
                    size: total,
                    risk,
                    reason,
                    suggested_command: cmd,
                    last_accessed_days: get_last_accessed_days(&path),
                });
            }
        }
    }

    recs
}

fn get_pyenv_default_version(home: &Path) -> Option<String> {
    let version_file = home.join(".pyenv/version");
    if version_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&version_file) {
            return Some(content.trim().to_string());
        }
    }
    None
}

fn check_python_ml_libs(python_dir: &Path) -> String {
    let ml_libs = [
        "torch", "tensorflow", "transformers", "jax",
        "diffusers", "onnxruntime", "sentencepiece",
    ];

    // Find site-packages directory
    let lib_dir = python_dir.join("lib");
    if !lib_dir.exists() {
        return String::new();
    }

    let mut found_libs = Vec::new();

    if let Ok(python_entries) = std::fs::read_dir(&lib_dir) {
        for entry in python_entries.flatten() {
            let entry_path = entry.path();
            if !entry_path.is_dir() {
                continue;
            }
            let dir_name = entry_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if dir_name.starts_with("python") {
                let site_packages = entry_path.join("site-packages");
                if site_packages.exists() {
                    for lib_name in &ml_libs {
                        let lib_path = site_packages.join(lib_name);
                        if lib_path.exists() {
                            if let Ok(size) = dir_size(&lib_path) {
                                if size > 10 * 1024 * 1024 {
                                    found_libs.push(format!("{} ({})", lib_name, human_bytes(size)));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    found_libs.join(", ")
}

fn scan_conda(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // Check ~/.conda/envs/
    let conda_envs = home.join(".conda/envs");
    if conda_envs.exists() {
        if let Ok(entries) = std::fs::read_dir(&conda_envs) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let env_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                if let Ok(total) = dir_size(&path) {
                    if total < min_size {
                        continue;
                    }

                    recs.push(Recommendation {
                        category: Category::Dev(DevKind::VersionManager),
                        path: path.display().to_string(),
                        size: total,
                        risk: Risk::Low,
                        reason: format!("Conda environment: {}", env_name),
                        suggested_command: format!("conda env remove -n {}", env_name),
                        last_accessed_days: get_last_accessed_days(&path),
                    });
                }
            }
        }
    }

    // Check conda package cache
    let conda_pkgs = home.join(".conda/pkgs");
    if conda_pkgs.exists() {
        if let Ok(total) = dir_size(&conda_pkgs) {
            if total > min_size {
                recs.push(Recommendation {
                    category: Category::Dev(DevKind::PackageCache),
                    path: conda_pkgs.display().to_string(),
                    size: total,
                    risk: Risk::Safe,
                    reason: format!("Conda package cache: {}", human_bytes(total)),
                    suggested_command: "conda clean --all".to_string(),
                    last_accessed_days: None,
                });
            }
        }
    }

    recs
}

// ── Rust ───────────────────────────────────────────────

fn scan_rustup(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let toolchains_dir = home.join(".rustup/toolchains");
    if !toolchains_dir.exists() {
        return vec![];
    }

    let current = get_rustup_default(home);
    let mut recs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&toolchains_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let toolchain = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let is_default = current.as_ref().map_or(false, |c| toolchain.starts_with(c));

            if let Ok(total) = dir_size(&path) {
                if total < min_size {
                    continue;
                }

                let (risk, reason) = if is_default {
                    (Risk::Low, format!("Rust toolchain: {} (default)", toolchain))
                } else {
                    (Risk::Safe, format!("Rust toolchain: {} (not default)", toolchain))
                };

                let cmd = if is_default {
                    format!("# Keep this toolchain (default): {}", toolchain)
                } else {
                    format!("rustup toolchain uninstall {}", toolchain)
                };

                recs.push(Recommendation {
                    category: Category::Dev(DevKind::VersionManager),
                    path: path.display().to_string(),
                    size: total,
                    risk,
                    reason,
                    suggested_command: cmd,
                    last_accessed_days: get_last_accessed_days(&path),
                });
            }
        }
    }

    recs
}

fn get_rustup_default(home: &Path) -> Option<String> {
    let settings_file = home.join(".rustup/settings.toml");
    if settings_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&settings_file) {
            for line in content.lines() {
                if line.starts_with("default_toolchain") {
                    if let Some(val) = line.split('=').nth(1) {
                        return Some(val.trim().trim_matches('"').to_string());
                    }
                }
            }
        }
    }
    None
}

// ── Helpers ────────────────────────────────────────────

fn extract_major_version(version: &str) -> Option<u32> {
    let cleaned = version.trim_start_matches('v');
    cleaned.split('.').next()?.parse().ok()
}

fn is_node_deprecated(major: Option<u32>) -> bool {
    match major {
        Some(v) if v < 18 => true,   // Node < 18 is EOL
        Some(18) => true,            // Node 18 EOL since 2025-04
        _ => false,
    }
}

fn get_last_accessed_days(path: &Path) -> Option<u64> {
    let metadata = std::fs::metadata(path).ok()?;
    let accessed = metadata.accessed().ok()?;
    let elapsed = accessed.elapsed().ok()?;
    Some(elapsed.as_secs() / 86400)
}
