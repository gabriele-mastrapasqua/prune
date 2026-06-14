use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Linux,
    Windows,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        }
    }

    pub fn cache_dir(&self) -> Option<PathBuf> {
        match self {
            Platform::MacOS => dirs::home_dir().map(|h| h.join("Library/Caches")),
            Platform::Linux => {
                std::env::var("XDG_CACHE_HOME")
                    .map(PathBuf::from)
                    .ok()
                    .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
            }
            Platform::Windows => dirs::cache_dir(),
        }
    }

    pub fn config_dir(&self) -> Option<PathBuf> {
        match self {
            Platform::MacOS => dirs::home_dir().map(|h| h.join("Library/Preferences")),
            Platform::Linux => {
                std::env::var("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .ok()
                    .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            }
            Platform::Windows => dirs::config_dir(),
        }
    }

    pub fn data_dir(&self) -> Option<PathBuf> {
        match self {
            Platform::MacOS => dirs::home_dir().map(|h| h.join("Library/Application Support")),
            Platform::Linux => {
                std::env::var("XDG_DATA_HOME")
                    .map(PathBuf::from)
                    .ok()
                    .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
            }
            Platform::Windows => dirs::data_dir(),
        }
    }

    pub fn log_dir(&self) -> Option<PathBuf> {
        match self {
            Platform::MacOS => dirs::home_dir().map(|h| h.join("Library/Logs")),
            Platform::Linux => {
                std::env::var("XDG_STATE_HOME")
                    .map(PathBuf::from)
                    .ok()
                    .or_else(|| dirs::home_dir().map(|h| h.join(".local/state")))
            }
            Platform::Windows => dirs::data_local_dir().map(|d| d.join("Temp")),
        }
    }

    pub fn home_dir(&self) -> Option<PathBuf> {
        dirs::home_dir()
    }
}

pub struct PlatformPaths {
    pub platform: Platform,
}

impl PlatformPaths {
    pub fn new() -> Self {
        Self {
            platform: Platform::current(),
        }
    }

    pub fn npm_cache(&self) -> Option<PathBuf> {
        self.platform.home_dir().map(|h| h.join(".npm/_cacache"))
    }

    pub fn pnpm_store(&self) -> Option<PathBuf> {
        match self.platform {
            Platform::MacOS => self.platform.home_dir().map(|h| h.join("Library/pnpm/store")),
            Platform::Linux | Platform::Windows => {
                self.platform.data_dir().map(|d| d.join("pnpm/store"))
            }
        }
    }

    pub fn yarn_cache(&self) -> Option<PathBuf> {
        match self.platform {
            Platform::MacOS => self.platform.home_dir().map(|h| h.join("Library/Caches/Yarn")),
            Platform::Linux => self.platform.cache_dir().map(|c| c.join("yarn")),
            Platform::Windows => self.platform.cache_dir().map(|c| c.join("Yarn")),
        }
    }

    pub fn pip_cache(&self) -> Option<PathBuf> {
        match self.platform {
            Platform::MacOS => self.platform.cache_dir().map(|c| c.join("pip")),
            Platform::Linux => self.platform.cache_dir().map(|c| c.join("pip")),
            Platform::Windows => self.platform.cache_dir().map(|c| c.join("pip")),
        }
    }

    pub fn uv_cache(&self) -> Option<PathBuf> {
        match self.platform {
            Platform::MacOS | Platform::Linux => {
                self.platform.cache_dir().map(|c| c.join("uv"))
            }
            Platform::Windows => self.platform.cache_dir().map(|c| c.join("uv")),
        }
    }

    pub fn cargo_registry(&self) -> Option<PathBuf> {
        self.platform.home_dir().map(|h| h.join(".cargo/registry/cache"))
    }

    pub fn go_mod_cache(&self) -> Option<PathBuf> {
        match self.platform {
            Platform::MacOS | Platform::Linux => {
                self.platform.home_dir().map(|h| h.join("go/pkg/mod/cache"))
            }
            Platform::Windows => {
                std::env::var("GOMODCACHE")
                    .map(PathBuf::from)
                    .ok()
                    .or_else(|| self.platform.home_dir().map(|h| h.join("go/pkg/mod/cache")))
            }
        }
    }

    pub fn homebrew_cache(&self) -> Option<PathBuf> {
        if self.platform == Platform::MacOS {
            self.platform.cache_dir().map(|c| c.join("Homebrew"))
        } else {
            None
        }
    }

    pub fn xcode_derived_data(&self) -> Option<PathBuf> {
        if self.platform == Platform::MacOS {
            self.platform.home_dir().map(|h| h.join("Library/Developer/Xcode/DerivedData"))
        } else {
            None
        }
    }

    pub fn docker_data(&self) -> Option<PathBuf> {
        match self.platform {
            Platform::MacOS => {
                self.platform.home_dir().map(|h| {
                    h.join("Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw")
                })
            }
            Platform::Linux => {
                self.platform.home_dir().map(|h| h.join(".docker/desktop/data/docker.raw"))
            }
            Platform::Windows => {
                self.platform.data_dir().map(|d| {
                    d.join("DockerDesktop/vms/0/data/Docker.raw")
                })
            }
        }
    }

    pub fn huggingface_cache(&self) -> Option<PathBuf> {
        self.platform.cache_dir().map(|c| c.join("huggingface/hub"))
    }

    pub fn ollama_models(&self) -> Option<PathBuf> {
        self.platform.home_dir().map(|h| h.join(".ollama/models"))
    }
}
