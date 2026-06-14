use crate::advisor::models::*;

pub fn format_report(recommendations: &[Recommendation]) -> String {
    if recommendations.is_empty() {
        return "✅ No recommendations found. Your disk looks clean!".to_string();
    }

    let mut output = String::new();
    output.push_str("═══════════════════════════════════════════\n");
    output.push_str("  DISK CLEANUP ADVISOR — macOS\n");
    output.push_str("═══════════════════════════════════════════\n");

    // Disk info
    if let Some(disk_info) = get_disk_info() {
        output.push_str(&format!("\n{}\n", disk_info));
    }

    let mut total_recoverable: u64 = 0;
    let mut safe_bytes: u64 = 0;
    let mut dev_bytes: u64 = 0;
    let mut review_bytes: u64 = 0;

    let mut current_category: Option<String> = None;
    for rec in recommendations {
        let cat_label = category_label(&rec.category);
        if current_category.as_ref() != Some(&cat_label) {
            output.push_str(&format!("\n── {} ──────────────────────\n", cat_label));
            current_category = Some(cat_label);
        }

        let risk_badge = match rec.risk {
            Risk::Safe => "✅ SAFE",
            Risk::Low => "🟡 LOW",
            Risk::Medium => "🟠 MEDIUM",
            Risk::Review => "🔴 REVIEW",
            Risk::Danger => "⛔ DANGER",
        };

        let age_info = match rec.last_accessed_days {
            Some(days) => format!(" | Not accessed: {}d", days),
            None => String::new(),
        };

        output.push_str(&format!(
            "  [!] {}\n      {} | {}{} \n      {}\n      → {}\n\n",
            rec.path,
            human_bytes(rec.size),
            risk_badge,
            age_info,
            rec.reason,
            rec.suggested_command,
        ));

        match rec.risk {
            Risk::Safe => safe_bytes += rec.size,
            Risk::Low => dev_bytes += rec.size,
            Risk::Medium | Risk::Review => review_bytes += rec.size,
            Risk::Danger => {}
        }
        total_recoverable += rec.size;
    }

    output.push_str("\n── 📋 Summary ──────────────────────────\n");
    output.push_str(&format!(
        "  Total recoverable:  {}\n",
        human_bytes(total_recoverable)
    ));
    output.push_str(&format!(
        "  Safe (no risk):     {}\n",
        human_bytes(safe_bytes)
    ));
    output.push_str(&format!(
        "  Dev-only:           {}\n",
        human_bytes(dev_bytes)
    ));
    output.push_str(&format!(
        "  Review required:    {}\n",
        human_bytes(review_bytes)
    ));

    output
}

pub fn format_json(recommendations: &[Recommendation]) -> String {
    serde_json::to_string_pretty(recommendations).unwrap_or_else(|_| "[]".to_string())
}

fn category_label(category: &Category) -> String {
    match category {
        Category::Cache => "🗑️ Cache & Temp".to_string(),
        Category::Log => "📋 Logs".to_string(),
        Category::Dev(kind) => match kind {
            DevKind::BuildArtifact => "🛠️ Dev Artifacts".to_string(),
            DevKind::PackageCache => "📦 Package Caches".to_string(),
            DevKind::VEnv => "🐍 Python Envs".to_string(),
            DevKind::Simulator => "📱 iOS Simulator".to_string(),
            DevKind::Docker => "🐳 Docker".to_string(),
            DevKind::VersionManager => "🔀 Version Managers".to_string(),
            DevKind::Xcode => "🔨 Xcode".to_string(),
            DevKind::Android => "🤖 Android".to_string(),
        },
        Category::AiModel => "🤖 AI Models".to_string(),
        Category::UserOldFiles => "📄 Large & Old Files".to_string(),
        Category::Installer => "💿 Downloads & Installers".to_string(),
        Category::SystemTemp => "🗑️ System Temp".to_string(),
        Category::Duplicate => "📑 Duplicate Files".to_string(),
        Category::Snapshot => "📸 Snapshots".to_string(),
        Category::Unknown => "❓ Unknown".to_string(),
    }
}

fn get_disk_info() -> Option<String> {
    // Use df to get disk info for root
    let output = std::process::Command::new("df")
        .args(["-h", "/"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() >= 2 {
        let parts: Vec<&str> = lines[1].split_whitespace().collect();
        if parts.len() >= 4 {
            return Some(format!(
                "Disk: {} total | Used: {} | Available: {}",
                parts[1], parts[2], parts[3]
            ));
        }
    }
    None
}
