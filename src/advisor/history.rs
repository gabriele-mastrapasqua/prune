use crate::advisor::models::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub timestamp: DateTime<Utc>,
    pub total_recoverable: u64,
    pub safe_bytes: u64,
    pub review_bytes: u64,
    pub by_category: HashMap<String, u64>,
    pub recommendation_count: usize,
}

pub fn get_history_path() -> PathBuf {
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    data_dir.join("prune").join("history.json")
}

pub fn load_history() -> Vec<ScanResult> {
    let path = get_history_path();
    if !path.exists() {
        return vec![];
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => vec![],
    }
}

pub fn save_scan_result(recommendations: &[Recommendation]) {
    let mut history = load_history();

    let mut by_category: HashMap<String, u64> = HashMap::new();
    let mut safe_bytes: u64 = 0;
    let mut review_bytes: u64 = 0;
    let mut total: u64 = 0;

    for rec in recommendations {
        total += rec.size;
        let cat_key = format!("{:?}", rec.category);
        *by_category.entry(cat_key).or_insert(0) += rec.size;

        match rec.risk {
            Risk::Safe => safe_bytes += rec.size,
            Risk::Review | Risk::Medium => review_bytes += rec.size,
            _ => {}
        }
    }

    let result = ScanResult {
        timestamp: Utc::now(),
        total_recoverable: total,
        safe_bytes,
        review_bytes,
        by_category,
        recommendation_count: recommendations.len(),
    };

    history.push(result);

    // Keep only last 90 entries
    if history.len() > 90 {
        history = history[history.len() - 90..].to_vec();
    }

    let path = get_history_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&history) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn format_sparkline(history: &[ScanResult]) -> String {
    if history.is_empty() {
        return "No scan history yet. Run a scan to start tracking.".to_string();
    }

    let values: Vec<u64> = history.iter().map(|r| r.total_recoverable).collect();
    let sparkline = sparkline_string(&values);

    let mut output = String::new();
    output.push_str("Disk Usage Trend (recoverable space):\n");
    output.push_str(&format!("  {}\n\n", sparkline));

    if let Some(first) = history.first() {
        output.push_str(&format!(
            "  First scan: {} — {}\n",
            first.timestamp.format("%Y-%m-%d %H:%M"),
            human_bytes(first.total_recoverable)
        ));
    }
    if let Some(last) = history.last() {
        output.push_str(&format!(
            "  Last scan:  {} — {}\n",
            last.timestamp.format("%Y-%m-%d %H:%M"),
            human_bytes(last.total_recoverable)
        ));
    }

    if history.len() >= 2 {
        let first_val = history.first().unwrap().total_recoverable;
        let last_val = history.last().unwrap().total_recoverable;
        let diff = last_val as i64 - first_val as i64;
        let sign = if diff > 0 { "+" } else { "" };
        output.push_str(&format!(
            "  Change:   {}{} ({} scans)\n",
            sign,
            human_bytes(diff.unsigned_abs()),
            history.len()
        ));
    }

    output
}

fn sparkline_string(values: &[u64]) -> String {
    const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    if values.is_empty() {
        return String::new();
    }

    let min_val = *values.iter().min().unwrap_or(&0);
    let max_val = *values.iter().max().unwrap_or(&1);
    let range = max_val.saturating_sub(min_val);

    values
        .iter()
        .map(|&v| {
            if range == 0 {
                BLOCKS[0]
            } else {
                let normalized = ((v - min_val) as f64 / range as f64 * 7.0) as usize;
                BLOCKS[normalized.min(7)]
            }
        })
        .collect()
}
