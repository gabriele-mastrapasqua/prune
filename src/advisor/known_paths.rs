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

    known
}

pub struct KnownPath {
    pub path: std::path::PathBuf,
    pub category: Category,
    pub risk: Risk,
    pub description: &'static str,
    pub suggested_command: &'static str,
}
