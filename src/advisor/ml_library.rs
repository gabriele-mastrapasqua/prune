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

    // Scan Ollama models (per-model granularity)
    recs.extend(scan_ollama_models(home, min_size));

    // Scan LM Studio models (per-GGUF granularity)
    recs.extend(scan_lmstudio_models(home, min_size));

    // Scan GPT4All models
    recs.extend(scan_gpt4all_models(home, min_size));

    // Scan text-generation-webui models
    recs.extend(scan_textgen_models(home, min_size));

    // Scan LocalAI models
    recs.extend(scan_localai_models(home, min_size));

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

fn scan_ollama_models(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    let ollama_dir = home.join(".ollama");
    let manifests_dir = ollama_dir.join("models/manifests");
    let blobs_dir = ollama_dir.join("models/blobs");

    if !manifests_dir.exists() || !blobs_dir.exists() {
        // Fallback: scan total directory
        if ollama_dir.exists() {
            if let Ok(total) = dir_size(&ollama_dir) {
                if total >= min_size {
                    recs.push(Recommendation {
                        category: Category::AiModel,
                        path: ollama_dir.display().to_string(),
                        size: total,
                        risk: Risk::Review,
                        reason: format!("Ollama models: {}", human_bytes(total)),
                        suggested_command: "ollama list && ollama rm <model>".to_string(),
                        last_accessed_days: get_last_accessed_days(&ollama_dir),
                    });
                }
            }
        }
        return recs;
    }

    // Walk manifests directory to find individual models
    // Structure: manifests/registry.ollama.ai/<namespace>/<model>/<tag>
    if let Ok(registry_entries) = std::fs::read_dir(&manifests_dir) {
        for registry_entry in registry_entries.flatten() {
            let registry_path = registry_entry.path();
            if !registry_path.is_dir() {
                continue;
            }

            if let Ok(ns_entries) = std::fs::read_dir(&registry_path) {
                for ns_entry in ns_entries.flatten() {
                    let ns_path = ns_entry.path();
                    if !ns_path.is_dir() {
                        continue;
                    }

                    if let Ok(model_entries) = std::fs::read_dir(&ns_path) {
                        for model_entry in model_entries.flatten() {
                            let model_path = model_entry.path();
                            if !model_path.is_dir() {
                                continue;
                            }

                            let model_name = model_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");

                            let ns_name = ns_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("library");

                            // Scan tags for this model
                            if let Ok(tag_entries) = std::fs::read_dir(&model_path) {
                                for tag_entry in tag_entries.flatten() {
                                    let tag_path = tag_entry.path();
                                    if !tag_path.is_file() {
                                        continue;
                                    }

                                    let tag_name = tag_path
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("latest");

                                    let full_name = format!("{}/{}:{}", ns_name, model_name, tag_name);

                                    // Read the manifest to find blob references
                                    let manifest_content = match std::fs::read_to_string(&tag_path) {
                                        Ok(c) => c,
                                        Err(_) => continue,
                                    };

                                    // Parse blob digests from manifest and sum their sizes
                                    let mut model_size = 0u64;
                                    for line in manifest_content.lines() {
                                        if line.contains("\"digest\"") {
                                            if let Some(digest_start) = line.find(": ") {
                                                let digest = line[digest_start + 2..]
                                                    .trim_matches('"')
                                                    .trim_matches(',');
                                                // Convert digest to blob path (sha256-xxx)
                                                let blob_name = digest.replace(':', "-");
                                                let blob_path = blobs_dir.join(format!(
                                                    "sha256-{}",
                                                    blob_name.trim_start_matches("sha256:")
                                                ));
                                                if blob_path.exists() {
                                                    if let Ok(meta) = std::fs::metadata(&blob_path) {
                                                        model_size += meta.len();
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if model_size >= min_size {
                                        let days = get_last_accessed_days(&tag_path);
                                        let risk = match days {
                                            Some(d) if d > 180 => Risk::Safe,
                                            Some(d) if d > 90 => Risk::Low,
                                            _ => Risk::Medium,
                                        };

                                        recs.push(Recommendation {
                                            category: Category::AiModel,
                                            path: tag_path.display().to_string(),
                                            size: model_size,
                                            risk,
                                            reason: format!(
                                                "Ollama model '{}': {}{}",
                                                full_name,
                                                human_bytes(model_size),
                                                days.map(|d| format!(" (not used in {} days)", d))
                                                    .unwrap_or_default()
                                            ),
                                            suggested_command: format!("ollama rm '{}'", full_name),
                                            last_accessed_days: days,
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

    recs
}

fn scan_lmstudio_models(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    let model_dirs = [
        home.join(".lmstudio/models"),
        home.join("Library/Application Support/LM Studio/models"),
    ];

    for dir in &model_dirs {
        if !dir.exists() {
            continue;
        }

        scan_gguf_files(dir, min_size, "LM Studio", &mut recs);
    }

    recs
}

fn scan_gpt4all_models(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    let model_dirs = [
        home.join("Library/Application Support/nomic.ai/GPT4All"),
        home.join(".local/share/nomic.ai/GPT4All"),
        home.join(".cache/gpt4all"),
    ];

    for dir in &model_dirs {
        if !dir.exists() {
            continue;
        }

        scan_gguf_files(dir, min_size, "GPT4All", &mut recs);
    }

    recs
}

fn scan_textgen_models(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    let model_dirs = [
        home.join("text-generation-webui/models"),
        home.join(".text-generation-webui/models"),
    ];

    for dir in &model_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                if let Ok(total) = dir_size(&path) {
                    if total >= min_size {
                        let model_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let days = get_last_accessed_days(&path);
                        let risk = match days {
                            Some(d) if d > 180 => Risk::Safe,
                            Some(d) if d > 90 => Risk::Low,
                            _ => Risk::Medium,
                        };

                        recs.push(Recommendation {
                            category: Category::AiModel,
                            path: path.display().to_string(),
                            size: total,
                            risk,
                            reason: format!(
                                "text-generation-webui model '{}': {}{}",
                                model_name,
                                human_bytes(total),
                                days.map(|d| format!(" (not used in {} days)", d))
                                    .unwrap_or_default()
                            ),
                            suggested_command: format!("rm -rf '{}'", path.display()),
                            last_accessed_days: days,
                        });
                    }
                }
            }
        }
    }

    recs
}

fn scan_localai_models(home: &Path, min_size: u64) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    let model_dirs = [
        home.join(".localai/models"),
        home.join("Library/Application Support/LocalAI/models"),
    ];

    for dir in &model_dirs {
        if !dir.exists() {
            continue;
        }

        scan_gguf_files(dir, min_size, "LocalAI", &mut recs);
    }

    recs
}

fn scan_gguf_files(dir: &Path, min_size: u64, tool_name: &str, recs: &mut Vec<Recommendation>) {
    if !dir.is_dir() {
        // Single directory with GGUF files
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext != "gguf" && ext != "bin" {
                    continue;
                }

                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() >= min_size {
                        let model_name = path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let days = get_last_accessed_days(&path);
                        let risk = match days {
                            Some(d) if d > 180 => Risk::Safe,
                            Some(d) if d > 90 => Risk::Low,
                            _ => Risk::Medium,
                        };

                        recs.push(Recommendation {
                            category: Category::AiModel,
                            path: path.display().to_string(),
                            size: meta.len(),
                            risk,
                            reason: format!(
                                "{} GGUF model '{}': {}{}",
                                tool_name,
                                model_name,
                                human_bytes(meta.len()),
                                days.map(|d| format!(" (not used in {} days)", d))
                                    .unwrap_or_default()
                            ),
                            suggested_command: format!("rm -f '{}'", path.display()),
                            last_accessed_days: days,
                        });
                    }
                }
            }
        }
        return;
    }

    // Directory with subdirectories containing GGUF files
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext != "gguf" && ext != "bin" {
                    continue;
                }

                if let Ok(meta) = std::fs::metadata(&path) {
                    if meta.len() >= min_size {
                        let model_name = path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let days = get_last_accessed_days(&path);
                        let risk = match days {
                            Some(d) if d > 180 => Risk::Safe,
                            Some(d) if d > 90 => Risk::Low,
                            _ => Risk::Medium,
                        };

                        recs.push(Recommendation {
                            category: Category::AiModel,
                            path: path.display().to_string(),
                            size: meta.len(),
                            risk,
                            reason: format!(
                                "{} model '{}': {}{}",
                                tool_name,
                                model_name,
                                human_bytes(meta.len()),
                                days.map(|d| format!(" (not used in {} days)", d))
                                    .unwrap_or_default()
                            ),
                            suggested_command: format!("rm -f '{}'", path.display()),
                            last_accessed_days: days,
                        });
                    }
                }
            } else if path.is_dir() {
                // Recurse one level for model subdirectories
                if let Ok(total) = dir_size(&path) {
                    if total >= min_size {
                        let model_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let days = get_last_accessed_days(&path);
                        let risk = match days {
                            Some(d) if d > 180 => Risk::Safe,
                            Some(d) if d > 90 => Risk::Low,
                            _ => Risk::Medium,
                        };

                        recs.push(Recommendation {
                            category: Category::AiModel,
                            path: path.display().to_string(),
                            size: total,
                            risk,
                            reason: format!(
                                "{} model '{}': {}{}",
                                tool_name,
                                model_name,
                                human_bytes(total),
                                days.map(|d| format!(" (not used in {} days)", d))
                                    .unwrap_or_default()
                            ),
                            suggested_command: format!("rm -rf '{}'", path.display()),
                            last_accessed_days: days,
                        });
                    }
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
