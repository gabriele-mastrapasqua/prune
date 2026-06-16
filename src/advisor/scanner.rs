use crate::advisor::cli_tools_scanner;
use crate::advisor::heuristics;
use crate::advisor::known_paths;
use crate::advisor::ml_library;
use crate::advisor::models::*;
use crate::advisor::package_manager;
use crate::advisor::rules;
use crate::advisor::safe_list;
use crate::advisor::update_scanner;
use crate::advisor::user_folders;
use crate::advisor::version_manager;
use std::path::{Path, PathBuf};

pub struct AdvisorEngine {
    pub min_size: u64,
    pub risk_limit: Risk,
    pub categories: Option<Vec<Category>>,
    pub no_dev: bool,
    pub no_ai: bool,
    pub older_than_days: Option<u64>,
}

impl Default for AdvisorEngine {
    fn default() -> Self {
        Self {
            min_size: 100 * 1024 * 1024, // 100 MB
            risk_limit: Risk::Review,
            categories: None,
            no_dev: false,
            no_ai: false,
            older_than_days: None,
        }
    }
}

impl AdvisorEngine {
    pub fn scan_home(&self) -> Vec<Recommendation> {
        let home = match std::env::var("HOME") {
            Ok(h) => PathBuf::from(h),
            Err(_) => return vec![],
        };

        let mut recommendations = Vec::new();

        // 1. Scan known macOS paths
        self.scan_known_paths(&home, &mut recommendations);

        // 2. Rule-based scanning (dead containers, installers, xcode)
        let rule_recs = rules::evaluate_rules(&home, self.no_dev, self.no_ai);
        recommendations.extend(rule_recs);

        // 3. Heuristic scanning (old large files)
        let heur_recs = heuristics::heuristic_analysis(&home, self.min_size, self.older_than_days);
        recommendations.extend(heur_recs);

        // 4. Scan common dev directories recursively (limited depth)
        self.scan_dev_dirs(&home, &mut recommendations);

        // 5. Scan version managers (nvm, pyenv, volta, fnm, conda, rustup)
        if !self.no_dev {
            let vm_recs = version_manager::scan_all_versions(&home, self.min_size);
            recommendations.extend(vm_recs);
        }

        // 6. Scan package manager caches and usage
        if !self.no_dev {
            let pm_recs = package_manager::scan_package_managers(&home, self.min_size);
            recommendations.extend(pm_recs);
        }

        // 7. Scan ML artifacts (models, CUDA, etc.)
        if !self.no_ai {
            let ml_recs = ml_library::scan_ml_artifacts(&home, self.min_size);
            recommendations.extend(ml_recs);
        }

        // 8. Scan auto-update residue (ShipIt, Sparkle, etc.)
        let update_recs = update_scanner::scan_update_residue(&home, self.min_size);
        recommendations.extend(update_recs);

        // 9. Scan dev/AI CLI tools (Claude Code, Gemini CLI, etc.)
        let cli_recs = cli_tools_scanner::scan_cli_tools(&home, self.min_size);
        recommendations.extend(cli_recs);

        // 10. Scan user folders (Downloads, Documents, etc.) for large/old files
        let folder_recs = user_folders::scan_user_folders_as_recommendations(&home, self.min_size);
        recommendations.extend(folder_recs);

        // Filter by risk, size, category
        recommendations.retain(|r| {
            if !self.risk_ok(&r.risk) {
                return false;
            }
            if r.size < self.min_size {
                return false;
            }
            if !safe_list::is_safe_to_suggest(&r.path) {
                return false;
            }
            if self.no_dev && matches!(r.category, Category::Dev(_)) {
                return false;
            }
            if self.no_ai && matches!(r.category, Category::AiModel) {
                return false;
            }
            if let Some(ref cats) = self.categories {
                if !cats.contains(&r.category) {
                    return false;
                }
            }
            true
        });

        // Deduplicate by path: merge categories, keep largest size, safest risk
        recommendations = deduplicate_recommendations(recommendations);

        // Sort by size descending
        recommendations.sort_by_key(|b| std::cmp::Reverse(b.size));
        recommendations
    }

    fn scan_known_paths(&self, _home: &Path, recs: &mut Vec<Recommendation>) {
        for known in known_paths::get_known_paths() {
            let path = known.path;

            // Skip if path doesn't exist
            if !path.exists() {
                continue;
            }

            // For directory known paths, compute total size
            if path.is_dir() {
                let total = match dir_size(&path) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if total >= self.min_size {
                    recs.push(Recommendation {
                        category: known.category.clone(),
                        path: path.display().to_string(),
                        size: total,
                        risk: known.risk.clone(),
                        reason: known.description.to_string(),
                        suggested_command: known.suggested_command.to_string(),
                        last_accessed_days: None,
                    });
                }
            } else if path.is_file() {
                let meta = match std::fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                if meta.len() >= self.min_size {
                    recs.push(Recommendation {
                        category: known.category.clone(),
                        path: path.display().to_string(),
                        size: meta.len(),
                        risk: known.risk.clone(),
                        reason: known.description.to_string(),
                        suggested_command: known.suggested_command.to_string(),
                        last_accessed_days: None,
                    });
                }
            }
        }
    }

    fn scan_dev_dirs(&self, home: &Path, recs: &mut Vec<Recommendation>) {
        // Scan common project-containing directories for build artifacts
        let scan_roots = [
            home.join("source"),
            home.join("projects"),
            home.join("code"),
            home.join("dev"),
            home.join("work"),
            home.join("repos"),
            home.join("Development"),
            home.join("Developer"),
        ];

        let patterns: &[(&str, Category, &str, Risk)] = &[
            (
                "node_modules",
                Category::Dev(DevKind::BuildArtifact),
                "Node.js dependencies",
                Risk::Safe,
            ),
            (
                "target",
                Category::Dev(DevKind::BuildArtifact),
                "Rust build artifacts",
                Risk::Safe,
            ),
            (
                ".venv",
                Category::Dev(DevKind::VEnv),
                "Python virtual environment",
                Risk::Safe,
            ),
            (
                "venv",
                Category::Dev(DevKind::VEnv),
                "Python virtual environment",
                Risk::Safe,
            ),
            (
                ".next",
                Category::Dev(DevKind::BuildArtifact),
                "Next.js build output",
                Risk::Safe,
            ),
            (
                ".nuxt",
                Category::Dev(DevKind::BuildArtifact),
                "Nuxt build output",
                Risk::Safe,
            ),
            (
                ".output",
                Category::Dev(DevKind::BuildArtifact),
                "Build output",
                Risk::Safe,
            ),
            (
                ".gradle",
                Category::Dev(DevKind::BuildArtifact),
                "Gradle cache",
                Risk::Safe,
            ),
        ];

        for root in &scan_roots {
            if !root.exists() || !root.is_dir() {
                continue;
            }
            // Only go 2 levels deep to avoid scanning entire trees
            if let Ok(entries) = std::fs::read_dir(root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    // Check for pattern dirs inside this project dir
                    if let Ok(sub_entries) = std::fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if !sub_path.is_dir() {
                                continue;
                            }
                            let dir_name =
                                sub_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                            for (pattern, category, reason, risk) in patterns {
                                if dir_name == *pattern {
                                    if let Ok(total) = dir_size(&sub_path) {
                                        if total >= self.min_size {
                                            let cmd = match *pattern {
                                                "node_modules" => {
                                                    format!("rm -rf '{}'", sub_path.display())
                                                }
                                                "target" => "cargo clean".to_string(),
                                                ".venv" | "venv" => {
                                                    format!("rm -rf '{}'", sub_path.display())
                                                }
                                                ".next" | ".nuxt" | ".output" => {
                                                    format!("rm -rf '{}'", sub_path.display())
                                                }
                                                ".gradle" => {
                                                    format!("rm -rf '{}'", sub_path.display())
                                                }
                                                _ => format!("rm -rf '{}'", sub_path.display()),
                                            };
                                            recs.push(Recommendation {
                                                category: category.clone(),
                                                path: sub_path.display().to_string(),
                                                size: total,
                                                risk: risk.clone(),
                                                reason: reason.to_string(),
                                                suggested_command: cmd,
                                                last_accessed_days: None,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn risk_ok(&self, risk: &Risk) -> bool {
        match risk {
            Risk::Safe => true,
            Risk::Low => self.risk_limit >= Risk::Low,
            Risk::Medium => self.risk_limit >= Risk::Medium,
            Risk::Review => self.risk_limit >= Risk::Review,
            Risk::Danger => false,
        }
    }
}

#[allow(dead_code)]
fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(path.replacen("~", &home, 1));
        }
    }
    PathBuf::from(path)
}

/// Deduplicate recommendations by path.
/// When duplicates exist, merge their categories, keep the largest size,
/// safest risk level, and combine reasons.
fn deduplicate_recommendations(recs: Vec<Recommendation>) -> Vec<Recommendation> {
    use std::collections::HashMap;

    let mut by_path: HashMap<String, Vec<Recommendation>> = HashMap::new();
    for rec in recs {
        by_path.entry(rec.path.clone()).or_default().push(rec);
    }

    let mut result = Vec::new();
    for (_path, group) in by_path {
        if group.len() == 1 {
            result.push(group.into_iter().next().unwrap());
            continue;
        }

        // Merge duplicates
        let merged = group.into_iter().reduce(|mut a, b| {
            // Keep the larger size
            if b.size > a.size {
                a.size = b.size;
            }

            // Keep the safest risk (lowest ordinal)
            if b.risk < a.risk {
                a.risk = b.risk;
            }

            // Merge categories: if different, use the more specific one
            // (prefer non-Unknown, prefer Cache over generic)
            if a.category != b.category {
                // If one is Unknown, use the other
                if matches!(a.category, Category::Unknown) {
                    a.category = b.category;
                }
                // Otherwise keep 'a' (first encountered, usually more specific)
            }

            // Merge reasons
            if a.reason != b.reason {
                a.reason = format!("{} | {}", a.reason, b.reason);
            }

            // Prefer the more actionable command
            if b.suggested_command.len() > a.suggested_command.len() {
                a.suggested_command = b.suggested_command;
            }

            // Keep the more recent last_accessed_days (smaller = more recent)
            match (a.last_accessed_days, b.last_accessed_days) {
                (Some(ad), Some(bd)) => {
                    a.last_accessed_days = Some(ad.min(bd));
                }
                (None, Some(_)) => {
                    a.last_accessed_days = b.last_accessed_days;
                }
                _ => {}
            }

            a
        });

        if let Some(rec) = merged {
            result.push(rec);
        }
    }

    result
}
