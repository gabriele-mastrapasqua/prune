use crate::advisor::models::*;
use std::path::Path;

pub fn scan_ml_artifacts(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // Scan HuggingFace hub for individual models
    recs.extend(scan_huggingface_models(home, min_size));

    // Scan for CUDA toolkit
    recs.extend(scan_cuda_toolkit(min_size));

    // Scan for large model files in common locations
    recs.extend(scan_model_files(home, min_size));

    recs
}

fn scan_huggingface_models(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let hf_hub = home.join(".cache/huggingface/hub");
    if !hf_hub.exists() {
        return vec![];
    }

    let mut recs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&hf_hub) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Only scan model/dataset directories (models--* or datasets--*)
            if !dir_name.starts_with("models--") && !dir_name.starts_with("datasets--") {
                continue;
            }

            if let Ok(total) = dir_size(&path) {
                if total < min_size {
                    continue;
                }

                // Parse org/model from directory name
                let model_name = dir_name
                    .trim_start_matches("models--")
                    .trim_start_matches("datasets--")
                    .replace("--", "/");

                let is_dataset = dir_name.starts_with("datasets--");
                let category_label = if is_dataset { "dataset" } else { "model" };

                let days = get_last_accessed_days(&path.join("blobs"));

                let (risk, reason) = match days {
                    Some(d) if d > 180 => (
                        Risk::Safe,
                        format!(
                            "HuggingFace {} '{}': {} (not accessed in {} days)",
                            category_label,
                            model_name,
                            human_bytes(total),
                            d
                        ),
                    ),
                    Some(d) if d > 90 => (
                        Risk::Low,
                        format!(
                            "HuggingFace {} '{}': {} (not accessed in {} days)",
                            category_label,
                            model_name,
                            human_bytes(total),
                            d
                        ),
                    ),
                    _ => (
                        Risk::Medium,
                        format!(
                            "HuggingFace {} '{}': {}",
                            category_label,
                            model_name,
                            human_bytes(total)
                        ),
                    ),
                };

                recs.push(Recommendation {
                    category: Category::AiModel,
                    path: path.display().to_string(),
                    size: total,
                    risk,
                    reason,
                    suggested_command: format!("rm -rf '{}'", path.display()),
                    last_accessed_days: days,
                });
            }
        }
    }

    recs
}

fn scan_cuda_toolkit(min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // Check /usr/local/cuda*
    if let Ok(entries) = std::fs::read_dir("/usr/local") {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if name.starts_with("cuda") && path.is_dir() {
                if let Ok(total) = dir_size(&path) {
                    if total > min_size {
                        let version = name.trim_start_matches("cuda-");
                        recs.push(Recommendation {
                            category: Category::AiModel,
                            path: path.display().to_string(),
                            size: total,
                            risk: Risk::Review,
                            reason: format!(
                                "CUDA Toolkit {}: {} (not needed on Mac)",
                                version,
                                human_bytes(total)
                            ),
                            suggested_command: format!("sudo rm -rf '{}'", path.display()),
                            last_accessed_days: get_last_accessed_days(&path),
                        });
                    }
                }
            }
        }
    }

    recs
}

fn scan_model_files(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // Scan common model storage locations for large files
    let model_dirs = [
        home.join("models"),
        home.join(".models"),
        home.join("MLModels"),
    ];

    for dir in &model_dirs {
        if !dir.exists() {
            continue;
        }

        if let Ok(total) = dir_size(dir) {
            if total > min_size {
                recs.push(Recommendation {
                    category: Category::AiModel,
                    path: dir.display().to_string(),
                    size: total,
                    risk: Risk::Review,
                    reason: format!("Model files directory: {}", human_bytes(total)),
                    suggested_command: format!("ls -lhS '{}'", dir.display()),
                    last_accessed_days: get_last_accessed_days(dir),
                });
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
