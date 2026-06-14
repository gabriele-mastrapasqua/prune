use crate::advisor::models::*;

/// Known macOS paths that commonly accumulate unnecessary data.
/// Each entry is checked by the advisor engine: if the path exists and exceeds
/// min_size, a recommendation is generated.
pub const KNOWN_PATHS: &[KnownPath] = &[
    // ── Caches ──────────────────────────────────────────
    KnownPath {
        path: "~/Library/Caches/Homebrew",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Homebrew download cache",
        suggested_command: "brew cleanup && rm -rf ~/Library/Caches/Homebrew/*",
    },
    KnownPath {
        path: "~/Library/Caches/com.apple.Safari/WebKitCache",
        category: Category::Cache,
        default_risk: Risk::Safe,
        description: "Safari WebKit cache",
        suggested_command: "rm -rf ~/Library/Caches/com.apple.Safari/WebKitCache/*",
    },
    KnownPath {
        path: "~/Library/Caches/pip",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Python pip cache",
        suggested_command: "pip cache purge",
    },
    KnownPath {
        path: "~/Library/Caches/com.apple.dt.Xcode",
        category: Category::Dev(DevKind::Xcode),
        default_risk: Risk::Safe,
        description: "Xcode cache",
        suggested_command: "rm -rf ~/Library/Caches/com.apple.dt.Xcode/*",
    },
    KnownPath {
        path: "~/Library/Caches/org.swift.swiftpm",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Swift Package Manager cache",
        suggested_command: "rm -rf ~/Library/Caches/org.swift.swiftpm/*",
    },
    KnownPath {
        path: "~/Library/Caches/CocoaPods",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "CocoaPods cache",
        suggested_command: "rm -rf ~/Library/Caches/CocoaPods/*",
    },
    KnownPath {
        path: "~/Library/Caches/Yarn",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Yarn v1 cache",
        suggested_command: "yarn cache clean",
    },
    KnownPath {
        path: "~/Library/Logs",
        category: Category::Log,
        default_risk: Risk::Safe,
        description: "Application logs",
        suggested_command: "rm -rf ~/Library/Logs/*",
    },

    // ── Package Manager Caches ──────────────────────────
    KnownPath {
        path: "~/.npm/_cacache",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "npm package cache",
        suggested_command: "npm cache clean --force",
    },
    KnownPath {
        path: "~/.npm/_npx",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "npx cache",
        suggested_command: "rm -rf ~/.npm/_npx/*",
    },
    KnownPath {
        path: "~/Library/pnpm/store",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "pnpm store",
        suggested_command: "pnpm store prune",
    },
    KnownPath {
        path: "~/.yarn/berry/cache",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Yarn Berry cache",
        suggested_command: "yarn cache clean",
    },
    // Bun cache is handled by package_manager scanner
    KnownPath {
        path: "~/.cache/uv",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "uv package cache",
        suggested_command: "uv cache clean",
    },
    KnownPath {
        path: "~/.cache/pypoetry",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Poetry cache",
        suggested_command: "poetry cache clear --all .",
    },
    KnownPath {
        path: "~/go/pkg/mod/cache",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Go module cache",
        suggested_command: "go clean -cache",
    },
    KnownPath {
        path: "~/.cache/go-build",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Go build cache",
        suggested_command: "go clean -cache",
    },
    KnownPath {
        path: "~/.cargo/registry/cache",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Cargo registry cache",
        suggested_command: "cargo cache --autoclean",
    },
    KnownPath {
        path: "~/.m2/repository",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Maven repository",
        suggested_command: "rm -rf ~/.m2/repository",
    },
    KnownPath {
        path: "~/.gradle/caches",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Gradle caches",
        suggested_command: "rm -rf ~/.gradle/caches",
    },
    KnownPath {
        path: "~/.composer/cache",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "PHP Composer cache",
        suggested_command: "composer clear-cache",
    },
    KnownPath {
        path: "~/.nuget/packages",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "NuGet global packages",
        suggested_command: "dotnet nuget locals all --clear",
    },
    KnownPath {
        path: "~/.pub-cache",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Dart pub cache",
        suggested_command: "dart pub cache clean",
    },

    // ── AI / ML Models ─────────────────────────────────
    KnownPath {
        path: "~/.ollama/models",
        category: Category::AiModel,
        default_risk: Risk::Review,
        description: "Ollama LLM models",
        suggested_command: "ollama list && ollama rm <model>",
    },
    KnownPath {
        path: "~/.cache/huggingface/hub",
        category: Category::AiModel,
        default_risk: Risk::Review,
        description: "HuggingFace model cache",
        suggested_command: "huggingface-cli delete-cache",
    },
    KnownPath {
        path: "~/.cache/torch",
        category: Category::AiModel,
        default_risk: Risk::Review,
        description: "PyTorch cache",
        suggested_command: "rm -rf ~/.cache/torch/*",
    },
    KnownPath {
        path: "~/.cache/whisper",
        category: Category::AiModel,
        default_risk: Risk::Review,
        description: "Whisper models",
        suggested_command: "rm -rf ~/.cache/whisper/*",
    },
    KnownPath {
        path: "~/.lmstudio/models",
        category: Category::AiModel,
        default_risk: Risk::Review,
        description: "LM Studio models",
        suggested_command: "lms unload --all && lms rm <model>",
    },
    KnownPath {
        path: "~/.cache/gpt4all",
        category: Category::AiModel,
        default_risk: Risk::Review,
        description: "GPT4All models",
        suggested_command: "rm -rf ~/.cache/gpt4all/*",
    },

    // ── IDE / Editor Caches ────────────────────────────
    KnownPath {
        path: "~/.vscode/extensions",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Review,
        description: "VS Code extensions",
        suggested_command: "code --list-extensions",
    },
    KnownPath {
        path: "~/.cursor/Cache",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Cursor cache",
        suggested_command: "rm -rf ~/.cursor/Cache/*",
    },

    // ── Version Managers ───────────────────────────────
    KnownPath {
        path: "~/.rustup/toolchains",
        category: Category::Dev(DevKind::VersionManager),
        default_risk: Risk::Low,
        description: "Rust toolchains",
        suggested_command: "rustup toolchain list && rustup toolchain uninstall <version>",
    },
    KnownPath {
        path: "~/.conda/pkgs",
        category: Category::Dev(DevKind::PackageCache),
        default_risk: Risk::Safe,
        description: "Conda package cache",
        suggested_command: "conda clean --all",
    },
];
