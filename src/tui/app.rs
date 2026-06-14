use std::collections::HashSet;

use crate::advisor::models::{Category, Recommendation, Risk};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Menu,
    Scanning,
    Results,
    Details(usize),
    ConfirmDeletion,
    Help,
    Quit,
}

pub struct App {
    pub mode: AppMode,
    pub menu_index: usize,
    pub menu_items: Vec<String>,
    pub recommendations: Vec<Recommendation>,
    pub selected_index: usize,
    pub selected_for_deletion: HashSet<usize>,
    pub confirmed_deletion: bool,
    pub scan_complete: bool,
    pub dev_mode: bool,
    pub min_size: u64,
    pub risk_limit: Risk,
    pub categories: Option<Vec<Category>>,
}

impl App {
    pub fn new(dev_mode: bool, min_size: u64, risk_limit: Risk) -> Self {
        Self {
            mode: AppMode::Menu,
            menu_index: 0,
            menu_items: vec![
                "Full Scan (DEV mode: ON)".to_string(),
                "Scan without DEV".to_string(),
                "Scan AI Models Only".to_string(),
                "View History".to_string(),
                "Settings".to_string(),
                "Help".to_string(),
            ],
            recommendations: Vec::new(),
            selected_index: 0,
            selected_for_deletion: HashSet::new(),
            confirmed_deletion: false,
            scan_complete: false,
            dev_mode,
            min_size,
            risk_limit,
            categories: None,
        }
    }

    pub fn total_recoverable(&self) -> u64 {
        self.recommendations.iter().map(|r| r.size).sum()
    }

    pub fn selected_count(&self) -> usize {
        self.selected_for_deletion.len()
    }

    pub fn selected_size(&self) -> u64 {
        self.selected_for_deletion
            .iter()
            .filter_map(|&i| self.recommendations.get(i))
            .map(|r| r.size)
            .sum()
    }
}
