use crate::advisor::models::{dir_size, human_bytes, Category, Recommendation, Risk};
use std::path::{Path, PathBuf};

pub fn scan_cli_tools(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    scan_dev_cli_tools(home, min_size, &mut recs);
    scan_ai_cli_tools(home, min_size, &mut recs);

    recs
}

fn scan_dev_cli_tools(home: &Path, min_size: u64, recs: &mut Vec<Recommendation>) {
    let mut tool_dirs: Vec<(PathBuf, &str, &str)> = vec![
        // Aider
        (
            home.join(".aider"),
            "Aider (AI coding assistant)",
            "rm -rf ~/.aider",
        ),
        // Continue
        (
            home.join(".continue"),
            "Continue (AI code assistant)",
            "rm -rf ~/.continue",
        ),
        // Copilot
        (
            home.join(".copilot"),
            "GitHub Copilot CLI",
            "rm -rf ~/.copilot",
        ),
        // Codex (OpenAI)
        (
            home.join(".codex"),
            "OpenAI Codex CLI",
            "rm -rf ~/.codex",
        ),
    ];

    if cfg!(target_os = "macos") {
        tool_dirs.push((
            home.join("Library/Application Support/Codex"),
            "OpenAI Codex data",
            "rm -rf ~/Library/Application\\ Support/Codex",
        ));
    } else if cfg!(target_os = "linux") {
        // XDG equivalents
        if let Some(xdg_data) = std::env::var("XDG_DATA_HOME").ok() {
            tool_dirs.push((
                PathBuf::from(xdg_data).join("Codex"),
                "OpenAI Codex data",
                "rm -rf ~/.local/share/Codex",
            ));
        } else {
            tool_dirs.push((
                home.join(".local/share/Codex"),
                "OpenAI Codex data",
                "rm -rf ~/.local/share/Codex",
            ));
        }
    }

    for (path, description, command) in &tool_dirs {
        if !path.exists() {
            continue;
        }

        if path.is_dir() {
            if let Ok(total) = dir_size(path) {
                if total >= min_size {
                    let days = get_last_accessed_days(path);
                    let risk = match days {
                        Some(d) if d > 90 => Risk::Safe,
                        Some(d) if d > 30 => Risk::Low,
                        _ => Risk::Medium,
                    };

                    recs.push(Recommendation {
                        category: Category::DevTool,
                        path: path.display().to_string(),
                        size: total,
                        risk,
                        reason: format!(
                            "{}: {}{}",
                            description,
                            human_bytes(total),
                            days.map(|d| format!(" (not accessed in {} days)", d))
                                .unwrap_or_default()
                        ),
                        suggested_command: command.to_string(),
                        last_accessed_days: days,
                    });
                }
            }

            // Also scan subdirectories for large items
            scan_subdirs_for_large(path, min_size, description, recs);
        }
    }
}

fn scan_ai_cli_tools(home: &Path, min_size: u64, recs: &mut Vec<Recommendation>) {
    let mut tool_dirs: Vec<(PathBuf, &str, &str)> = vec![
        // Claude Code (Anthropic)
        (
            home.join(".claude"),
            "Claude Code (Anthropic)",
            "rm -rf ~/.claude",
        ),
        // Gemini CLI (Google)
        (
            home.join(".gemini"),
            "Gemini CLI (Google)",
            "rm -rf ~/.gemini",
        ),
        // Qwen Code (Alibaba)
        (
            home.join(".qwen"),
            "Qwen Code",
            "rm -rf ~/.qwen",
        ),
        // OpenCode
        (
            home.join(".opencode"),
            "OpenCode",
            "rm -rf ~/.opencode",
        ),
    ];

    if cfg!(target_os = "macos") {
        tool_dirs.extend(vec![
            (
                home.join("Library/Application Support/Claude"),
                "Claude Code data",
                "rm -rf ~/Library/Application\\ Support/Claude",
            ),
            (
                home.join("Library/Caches/Claude"),
                "Claude Code cache",
                "rm -rf ~/Library/Caches/Claude",
            ),
            (
                home.join("Library/Application Support/Gemini"),
                "Gemini CLI data",
                "rm -rf ~/Library/Application\\ Support/Gemini",
            ),
            (
                home.join("Library/Application Support/gemini-cli"),
                "Gemini CLI data",
                "rm -rf ~/Library/Application\\ Support/gemini-cli",
            ),
            (
                home.join("Library/Application Support/Qwen"),
                "Qwen Code data",
                "rm -rf ~/Library/Application\\ Support/Qwen",
            ),
            (
                home.join("Library/Application Support/OpenCode"),
                "OpenCode data",
                "rm -rf ~/Library/Application\\ Support/OpenCode",
            ),
        ]);
    } else if cfg!(target_os = "linux") {
        // XDG equivalents on Linux
        let xdg_data = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".local/share"));
        let xdg_cache = std::env::var("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".cache"));

        tool_dirs.extend(vec![
            (xdg_data.join("claude"), "Claude Code data", "rm -rf ~/.local/share/claude"),
            (xdg_cache.join("claude"), "Claude Code cache", "rm -rf ~/.cache/claude"),
            (xdg_data.join("gemini"), "Gemini CLI data", "rm -rf ~/.local/share/gemini"),
            (xdg_data.join("gemini-cli"), "Gemini CLI data", "rm -rf ~/.local/share/gemini-cli"),
            (xdg_data.join("qwen"), "Qwen Code data", "rm -rf ~/.local/share/qwen"),
            (xdg_data.join("opencode"), "OpenCode data", "rm -rf ~/.local/share/opencode"),
        ]);
    }

    for (path, description, command) in &tool_dirs {
        if !path.exists() {
            continue;
        }

        if path.is_dir() {
            if let Ok(total) = dir_size(path) {
                if total >= min_size {
                    let days = get_last_accessed_days(path);
                    let risk = match days {
                        Some(d) if d > 90 => Risk::Safe,
                        Some(d) if d > 30 => Risk::Low,
                        _ => Risk::Medium,
                    };

                    recs.push(Recommendation {
                        category: Category::DevTool,
                        path: path.display().to_string(),
                        size: total,
                        risk,
                        reason: format!(
                            "{}: {}{}",
                            description,
                            human_bytes(total),
                            days.map(|d| format!(" (not accessed in {} days)", d))
                                .unwrap_or_default()
                        ),
                        suggested_command: command.to_string(),
                        last_accessed_days: days,
                    });
                }
            }

            // Scan subdirectories for large cached data
            scan_subdirs_for_large(path, min_size, description, recs);
        }
    }

    // Scan VS Code extensions for AI tool caches
    scan_vscode_ai_extensions(home, min_size, recs);
}

fn scan_vscode_ai_extensions(home: &Path, min_size: u64, recs: &mut Vec<Recommendation>) {
    let vscode_extensions = home.join(".vscode/extensions");
    if !vscode_extensions.exists() {
        return;
    }

    let ai_extension_prefixes = [
        "anthropic.claude",
        "github.copilot",
        "continue.continue",
        "openai.codex",
        "google.gemini",
    ];

    let entries = match std::fs::read_dir(&vscode_extensions) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_ai_ext = ai_extension_prefixes.iter().any(|prefix| name.starts_with(prefix));

        if !is_ai_ext {
            continue;
        }

        if let Ok(total) = dir_size(&path) {
            if total >= min_size {
                recs.push(Recommendation {
                    category: Category::DevTool,
                    path: path.display().to_string(),
                    size: total,
                    risk: Risk::Review,
                    reason: format!(
                        "VS Code AI extension '{}': {} (may contain cached models/data)",
                        name,
                        human_bytes(total)
                    ),
                    suggested_command: format!("code --uninstall-extension '{}'", name),
                    last_accessed_days: get_last_accessed_days(&path),
                });
            }
        }
    }
}

fn scan_subdirs_for_large(
    parent: &Path,
    min_size: u64,
    parent_desc: &str,
    recs: &mut Vec<Recommendation>,
) {
    let entries = match std::fs::read_dir(parent) {
        Ok(e) => e,
        Err(_) => return,
    };

    let cache_dir_names = ["cache", "caches", "tmp", "temp", "sessions", "logs", "history"];

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if cache_dir_names.iter().any(|c| dir_name.contains(c)) {
            if let Ok(total) = dir_size(&path) {
                if total >= min_size {
                    recs.push(Recommendation {
                        category: Category::DevTool,
                        path: path.display().to_string(),
                        size: total,
                        risk: Risk::Safe,
                        reason: format!(
                            "{} cache directory '{}': {}",
                            parent_desc,
                            dir_name,
                            human_bytes(total)
                        ),
                        suggested_command: format!("rm -rf '{}'", path.display()),
                        last_accessed_days: get_last_accessed_days(&path),
                    });
                }
            }
        }
    }
}

fn get_last_accessed_days(path: &Path) -> Option<u64> {
    let metadata = std::fs::metadata(path).ok()?;
    let accessed = metadata.accessed().ok()?;
    let elapsed = accessed.elapsed().ok()?;
    Some(elapsed.as_secs() / 86400)
}
