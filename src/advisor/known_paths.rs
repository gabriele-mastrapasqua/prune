use crate::advisor::models::*;
use crate::advisor::paths::PlatformPaths;

pub fn get_known_paths() -> Vec<KnownPath> {
    let paths = PlatformPaths::new();
    let mut known = Vec::new();

    // Package manager caches (cross-platform)
    if let Some(p) = paths.npm_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "npm cache",
            suggested_command: "npm cache clean --force",
        });
    }

    if let Some(p) = paths.pnpm_store() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "pnpm store",
            suggested_command: "pnpm store prune",
        });
    }

    if let Some(p) = paths.yarn_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "Yarn cache",
            suggested_command: "yarn cache clean",
        });
    }

    if let Some(p) = paths.pip_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "pip cache",
            suggested_command: "pip cache purge",
        });
    }

    if let Some(p) = paths.uv_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "uv cache",
            suggested_command: "uv cache clean",
        });
    }

    if let Some(p) = paths.cargo_registry() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "Cargo registry cache",
            suggested_command: "cargo cache --autoclean",
        });
    }

    if let Some(p) = paths.go_mod_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "Go module cache",
            suggested_command: "go clean -modcache",
        });
    }

    // macOS-specific
    if let Some(p) = paths.homebrew_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "Homebrew cache",
            suggested_command: "brew cleanup --prune=all",
        });
    }

    if let Some(p) = paths.xcode_derived_data() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "Xcode DerivedData",
            suggested_command: "rm -rf ~/Library/Developer/Xcode/DerivedData/*",
        });
    }

    // AI/ML caches (cross-platform)
    if let Some(p) = paths.huggingface_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::AiModel,
            risk: Risk::Review,
            description: "HuggingFace model cache",
            suggested_command: "rm -rf ~/.cache/huggingface/hub/*",
        });
    }

    if let Some(p) = paths.ollama_models() {
        known.push(KnownPath {
            path: p,
            category: Category::AiModel,
            risk: Risk::Review,
            description: "Ollama models",
            suggested_command: "ollama rm <model>",
        });
    }

    // Docker (cross-platform with platform-specific paths)
    if let Some(p) = paths.docker_data() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Review,
            description: "Docker data",
            suggested_command: "docker system prune -a --volumes",
        });
    }

    // Linux-specific
    if let Some(p) = paths.apt_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "APT package cache",
            suggested_command: "sudo apt-get clean",
        });
    }

    if let Some(p) = paths.snap_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "Snap package cache",
            suggested_command: "sudo snap set system refresh.retain=2",
        });
    }

    if let Some(p) = paths.flatpak_runtime() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Low,
            description: "Flatpak unused runtimes",
            suggested_command: "flatpak uninstall --unused",
        });
    }

    if let Some(p) = paths.journal_logs() {
        known.push(KnownPath {
            path: p,
            category: Category::Log,
            risk: Risk::Safe,
            description: "systemd journal logs",
            suggested_command: "sudo journalctl --vacuum-size=100M",
        });
    }

    if let Some(p) = paths.system_logs() {
        known.push(KnownPath {
            path: p,
            category: Category::Log,
            risk: Risk::Review,
            description: "System log files",
            suggested_command: "sudo find /var/log -name '*.gz' -delete",
        });
    }

    if let Some(p) = paths.pacman_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "Pacman package cache",
            suggested_command: "sudo pacman -Sc",
        });
    }

    if let Some(p) = paths.dnf_cache() {
        known.push(KnownPath {
            path: p,
            category: Category::Cache,
            risk: Risk::Safe,
            description: "DNF package cache",
            suggested_command: "sudo dnf clean all",
        });
    }

    known
}

pub struct KnownPath {
    pub path: std::path::PathBuf,
    pub category: Category,
    pub risk: Risk,
    pub description: &'static str,
    pub suggested_command: &'static str,
}
