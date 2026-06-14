use crate::advisor::models::*;
use crate::advisor::paths::PlatformPaths;
use std::path::Path;

pub fn scan_package_managers(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();
    let paths = PlatformPaths::new();

    // Check for package manager caches (cross-platform)
    recs.extend(check_pnpm(&paths, min_size));
    recs.extend(check_yarn(&paths, min_size));
    recs.extend(check_bun(home, min_size));
    recs.extend(check_uv_tools(home, min_size));

    recs
}

fn check_pnpm(paths: &PlatformPaths, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // pnpm store (cross-platform)
    if let Some(pnpm_store) = paths.pnpm_store() {
        if pnpm_store.exists() {
            if let Ok(total) = dir_size(&pnpm_store) {
                if total > min_size {
                    recs.push(Recommendation {
                        category: Category::Dev(DevKind::PackageCache),
                        path: pnpm_store.display().to_string(),
                        size: total,
                        risk: Risk::Safe,
                        reason: format!("pnpm content-addressable store: {}", human_bytes(total)),
                        suggested_command: "pnpm store prune".to_string(),
                        last_accessed_days: get_last_accessed_days(&pnpm_store),
                    });
                }
            }
        }
    }

    // pnpm global (cross-platform)
    if let Some(home) = paths.platform.home_dir() {
        let pnpm_global = if cfg!(target_os = "windows") {
            home.join("AppData\\Local\\pnpm\\global")
        } else {
            home.join(".local/share/pnpm/global")
        };

        if pnpm_global.exists() {
            if let Ok(total) = dir_size(&pnpm_global) {
                if total > min_size {
                    recs.push(Recommendation {
                        category: Category::Dev(DevKind::PackageCache),
                        path: pnpm_global.display().to_string(),
                        size: total,
                        risk: Risk::Low,
                        reason: format!("pnpm global packages: {}", human_bytes(total)),
                        suggested_command: "pnpm ls -g && pnpm rm -g <pkg>".to_string(),
                        last_accessed_days: get_last_accessed_days(&pnpm_global),
                    });
                }
            }
        }
    }

    recs
}

fn check_yarn(paths: &PlatformPaths, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // Yarn cache (cross-platform)
    if let Some(yarn_cache) = paths.yarn_cache() {
        if yarn_cache.exists() {
            if let Ok(total) = dir_size(&yarn_cache) {
                if total > min_size {
                    recs.push(Recommendation {
                        category: Category::Dev(DevKind::PackageCache),
                        path: yarn_cache.display().to_string(),
                        size: total,
                        risk: Risk::Safe,
                        reason: format!("Yarn cache: {}", human_bytes(total)),
                        suggested_command: "yarn cache clean".to_string(),
                        last_accessed_days: get_last_accessed_days(&yarn_cache),
                    });
                }
            }
        }
    }

    // Yarn Berry cache (cross-platform)
    if let Some(home) = paths.platform.home_dir() {
        let yarn_berry = home.join(".yarn/berry/cache");
        if yarn_berry.exists() {
            if let Ok(total) = dir_size(&yarn_berry) {
                if total > min_size {
                    recs.push(Recommendation {
                        category: Category::Dev(DevKind::PackageCache),
                        path: yarn_berry.display().to_string(),
                        size: total,
                        risk: Risk::Safe,
                        reason: format!("Yarn Berry cache: {}", human_bytes(total)),
                        suggested_command: "yarn cache clean".to_string(),
                        last_accessed_days: get_last_accessed_days(&yarn_berry),
                    });
                }
            }
        }
    }

    recs
}

fn check_bun(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // Bun cache (cross-platform)
    let bun_cache = home.join(".bun/install/cache");
    if bun_cache.exists() {
        if let Ok(total) = dir_size(&bun_cache) {
            if total > min_size {
                recs.push(Recommendation {
                    category: Category::Dev(DevKind::PackageCache),
                    path: bun_cache.display().to_string(),
                    size: total,
                    risk: Risk::Safe,
                    reason: format!("Bun install cache: {}", human_bytes(total)),
                    suggested_command: "rm -rf ~/.bun/install/cache/*".to_string(),
                    last_accessed_days: get_last_accessed_days(&bun_cache),
                });
            }
        }
    }

    recs
}

fn check_uv_tools(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // uv tools (cross-platform)
    let uv_tools = if cfg!(target_os = "windows") {
        home.join("AppData\\Local\\uv\\tools")
    } else {
        home.join(".local/share/uv/tools")
    };

    if !uv_tools.exists() {
        return recs;
    }

    if let Ok(entries) = std::fs::read_dir(&uv_tools) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let tool_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if let Ok(total) = dir_size(&path) {
                if total > min_size {
                    recs.push(Recommendation {
                        category: Category::Dev(DevKind::PackageCache),
                        path: path.display().to_string(),
                        size: total,
                        risk: Risk::Low,
                        reason: format!("uv tool: {} ({})", tool_name, human_bytes(total)),
                        suggested_command: format!("uv tool uninstall {}", tool_name),
                        last_accessed_days: get_last_accessed_days(&path),
                    });
                }
            }
        }
    }

    recs
}

fn get_last_accessed_days(path: &Path) -> Option<u64> {
    let metadata = std::fs::metadata(path).ok()?;
    let accessed = metadata.accessed().ok()?;
    let elapsed = accessed.elapsed().ok()?;
    Some(elapsed.as_secs() / 86400)
}
