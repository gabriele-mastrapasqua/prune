use std::collections::HashSet;

use crate::advisor::models::{AppInfo, Category, FolderSummary, Recommendation, Risk};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Menu,
    Scanning,
    Dashboard,
    Results,
    Details(usize),
    AppManager,
    DiskAnalyzer,
    ConfirmDeletion,
    Help,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmTarget {
    Recommendations,
    Apps,
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
    pub show_sidebar: bool,
    pub apps: Vec<AppInfo>,
    pub app_selected_index: usize,
    pub apps_for_uninstall: HashSet<usize>,
    pub confirm_target: ConfirmTarget,
    pub scan_frame: usize,
    pub scan_cancelled: bool,
    pub folder_summaries: Vec<FolderSummary>,
    pub disk_selected_index: usize,
    pub home_summary: Option<FolderSummary>,
    pub disk_scan_complete: bool,
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
                "Disk Analyzer".to_string(),
                "App Manager".to_string(),
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
            show_sidebar: true,
            apps: Vec::new(),
            app_selected_index: 0,
            apps_for_uninstall: HashSet::new(),
            confirm_target: ConfirmTarget::Recommendations,
            scan_frame: 0,
            scan_cancelled: false,
            folder_summaries: Vec::new(),
            disk_selected_index: 0,
            home_summary: None,
            disk_scan_complete: false,
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

    pub fn apps_total_size(&self) -> u64 {
        self.apps.iter().map(|a| a.total_size).sum()
    }

    pub fn apps_uninstall_size(&self) -> u64 {
        self.apps_for_uninstall
            .iter()
            .filter_map(|&i| self.apps.get(i))
            .map(|a| a.total_size)
            .sum()
    }
}
