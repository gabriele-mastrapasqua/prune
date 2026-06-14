use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[allow(dead_code)]
pub enum Category {
    Cache,
    Log,
    Dev(DevKind),
    AiModel,
    UserOldFiles,
    Installer,
    SystemTemp,
    Duplicate,
    Snapshot,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[allow(dead_code)]
pub enum DevKind {
    BuildArtifact,
    PackageCache,
    VEnv,
    Simulator,
    Docker,
    VersionManager,
    Xcode,
    Android,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[allow(dead_code)]
pub enum Risk {
    Safe,
    Low,
    Medium,
    Review,
    Danger,
}

#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    pub category: Category,
    pub path: String,
    pub size: u64,
    pub risk: Risk,
    pub reason: String,
    pub suggested_command: String,
    pub last_accessed_days: Option<u64>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KnownPath {
    pub path: &'static str,
    pub category: Category,
    pub default_risk: Risk,
    pub description: &'static str,
    pub suggested_command: &'static str,
}

pub fn human_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let idx = (bytes as f64).log(1024.0).floor() as usize;
    let idx = idx.min(UNITS.len() - 1);
    let value = bytes as f64 / 1024.0f64.powi(idx as i32);
    format!("{:.1} {}", value, UNITS[idx])
}

pub fn dir_size(path: &std::path::Path) -> Result<u64, std::io::Error> {
    let mut total = 0u64;
    if path.is_symlink() {
        return Ok(0);
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() && !entry.path().is_symlink() {
            total += dir_size(&entry.path())?;
        } else {
            total += metadata.len();
        }
    }
    Ok(total)
}
