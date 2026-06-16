pub mod app;
pub mod views;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::advisor::models::{Category, FolderSummary, Recommendation, Risk};
use crate::advisor::scanner::AdvisorEngine;
use crate::advisor::user_folders;
use app::{App, AppMode};

type ScanHandle = std::thread::JoinHandle<()>;
type ScanReceiver = std::sync::mpsc::Receiver<Vec<Recommendation>>;
type DiskHandle = std::thread::JoinHandle<()>;
type DiskReceiver = std::sync::mpsc::Receiver<(Vec<FolderSummary>, Option<FolderSummary>)>;

pub fn run(dev_mode: bool, min_size: Option<&str>, risk_limit: Option<&str>) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Parse options
    let min_size_bytes = min_size.and_then(parse_size).unwrap_or(100 * 1024 * 1024); // default 100 MB

    let risk = risk_limit.map(parse_risk).unwrap_or(Risk::Review);

    // Create app
    let mut app = App::new(dev_mode, min_size_bytes, risk);

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // If user selected items for deletion, execute them
    if app.confirmed_deletion {
        if !app.selected_for_deletion.is_empty() {
            execute_deletions(&app);
        }
        if !app.apps_for_uninstall.is_empty() {
            execute_app_uninstalls(&app);
        }
    }

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    use std::time::Duration;

    let mut _scan_handle: Option<ScanHandle> = None;
    let mut scan_rx: Option<ScanReceiver> = None;
    let mut _disk_handle: Option<DiskHandle> = None;
    let mut disk_rx: Option<DiskReceiver> = None;

    loop {
        terminal.draw(|f| match app.mode {
            AppMode::Menu => views::menu::render(f, app),
            AppMode::Scanning => views::scan::render(f, app),
            AppMode::Dashboard => views::dashboard::render(f, app),
            AppMode::Results => views::results::render(f, app),
            AppMode::Details(idx) => views::details::render(f, app, idx),
            AppMode::AppManager => views::apps::render(f, app),
            AppMode::DiskAnalyzer => views::disk_analyzer::render(f, app),
            AppMode::ConfirmDeletion => views::confirm::render(f, app),
            AppMode::Help => views::help::render(f, app),
            AppMode::Quit => {}
        })?;

        if matches!(app.mode, AppMode::Quit) {
            return Ok(());
        }

        // Animate spinner during scan
        if matches!(app.mode, AppMode::Scanning) && !app.scan_complete {
            app.scan_frame = (app.scan_frame + 1) % 20;
        }

        // Animate spinner during disk analysis
        if matches!(app.mode, AppMode::DiskAnalyzer) && !app.disk_scan_complete {
            app.scan_frame = (app.scan_frame + 1) % 10;
        }

        // Check for scan results from background thread
        if let Some(ref rx) = scan_rx {
            if let Ok(recs) = rx.try_recv() {
                app.recommendations = recs;
                app.scan_complete = true;
                scan_rx = None;
                _scan_handle = None;
            }
        }

        // Check for disk analyzer results from background thread
        if let Some(ref rx) = disk_rx {
            if let Ok((folders, home)) = rx.try_recv() {
                app.folder_summaries = folders;
                app.home_summary = home;
                app.disk_scan_complete = true;
                disk_rx = None;
                _disk_handle = None;
            }
        }

        // Auto-start disk scan when entering DiskAnalyzer mode
        if matches!(app.mode, AppMode::DiskAnalyzer) && !app.disk_scan_complete && disk_rx.is_none() {
            let (handle, rx) = start_disk_scan(app);
            _disk_handle = Some(handle);
            disk_rx = Some(rx);
        }

        // Non-blocking event read with timeout for animation
        let event_available = event::poll(Duration::from_millis(100))?;

        if event_available {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.mode {
                    AppMode::Menu => {
                        if let Some((handle, rx)) = handle_menu_input(app, key.code) {
                            _scan_handle = Some(handle);
                            scan_rx = Some(rx);
                        }
                    }
                    AppMode::Scanning => {
                        // If scan is running in background, mark for cancellation
                        if !app.scan_complete {
                            app.scan_cancelled = true;
                            app.mode = AppMode::Menu;
                            _scan_handle = None;
                            scan_rx = None;
                        } else {
                            handle_scan_input(app, key.code);
                        }
                    }
                    AppMode::Dashboard => handle_dashboard_input(app, key.code),
                    AppMode::Results => handle_results_input(app, key.code),
                    AppMode::Details(idx) => handle_details_input(app, key.code, idx),
                    AppMode::AppManager => handle_apps_input(app, key.code),
                    AppMode::DiskAnalyzer => handle_disk_analyzer_input(app, key.code),
                    AppMode::ConfirmDeletion => handle_confirm_input(app, key.code),
                    AppMode::Help => handle_help_input(app, key.code),
                    AppMode::Quit => {}
                }
            }
        }

        // If scanning, check if done
        if matches!(app.mode, AppMode::Scanning) && app.scan_complete {
            app.mode = AppMode::Dashboard;
        }
    }
}

fn handle_menu_input(app: &mut App, key: KeyCode) -> Option<(ScanHandle, ScanReceiver)> {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.mode = AppMode::Quit,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.menu_index > 0 {
                app.menu_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.menu_index < app.menu_items.len() - 1 {
                app.menu_index += 1;
            }
        }
        KeyCode::Enter => {
            match app.menu_index {
                0 => {
                    // Full scan with DEV mode
                    app.dev_mode = true;
                    return Some(start_scan(app));
                }
                1 => {
                    // Scan without DEV
                    app.dev_mode = false;
                    return Some(start_scan(app));
                }
                2 => {
                    // Scan AI only
                    app.dev_mode = false;
                    app.categories = Some(vec![Category::AiModel]);
                    return Some(start_scan(app));
                }
                3 => {
                    // Disk Analyzer
                    app.disk_scan_complete = false;
                    app.folder_summaries.clear();
                    app.home_summary = None;
                    app.disk_selected_index = 0;
                    app.mode = AppMode::DiskAnalyzer;
                }
                4 => {
                    // App Manager
                    app.apps = crate::advisor::app_scanner::scan_applications();
                    app.app_selected_index = 0;
                    app.apps_for_uninstall.clear();
                    app.mode = AppMode::AppManager;
                }
                5 => {
                    // View history (TODO)
                }
                6 => {
                    // Settings (TODO)
                }
                7 => {
                    app.mode = AppMode::Help;
                }
                _ => {}
            }
        }
        _ => {}
    }
    None
}

fn handle_scan_input(app: &mut App, key: KeyCode) {
    if matches!(key, KeyCode::Char('q') | KeyCode::Esc) {
        app.mode = AppMode::Menu;
    }
}

fn handle_dashboard_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.mode = AppMode::Menu,
        KeyCode::Enter | KeyCode::Tab => app.mode = AppMode::Results,
        KeyCode::Char('?') => app.mode = AppMode::Help,
        _ => {}
    }
}

fn handle_apps_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.mode = AppMode::Menu,
        KeyCode::Tab => {
            app.show_sidebar = !app.show_sidebar;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.app_selected_index > 0 {
                app.app_selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.app_selected_index < app.apps.len().saturating_sub(1) {
                app.app_selected_index += 1;
            }
        }
        KeyCode::Char(' ') => {
            if app.apps_for_uninstall.contains(&app.app_selected_index) {
                app.apps_for_uninstall.remove(&app.app_selected_index);
            } else {
                app.apps_for_uninstall.insert(app.app_selected_index);
            }
        }
        KeyCode::Char('c') => {
            if !app.apps_for_uninstall.is_empty() {
                app.confirm_target = app::ConfirmTarget::Apps;
                app.mode = AppMode::ConfirmDeletion;
            }
        }
        KeyCode::Char('?') => {
            app.mode = AppMode::Help;
        }
        _ => {}
    }
}

fn handle_disk_analyzer_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.mode = AppMode::Menu,
        KeyCode::Tab => {
            app.show_sidebar = !app.show_sidebar;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.disk_selected_index > 0 {
                app.disk_selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.disk_selected_index < app.folder_summaries.len().saturating_sub(1) {
                app.disk_selected_index += 1;
            }
        }
        _ => {}
    }
}

fn handle_results_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.mode = AppMode::Menu,
        KeyCode::Tab => {
            app.show_sidebar = !app.show_sidebar;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_index < app.recommendations.len().saturating_sub(1) {
                app.selected_index += 1;
            }
        }
        KeyCode::Char(' ') => {
            // Toggle selection
            if app.selected_for_deletion.contains(&app.selected_index) {
                app.selected_for_deletion.remove(&app.selected_index);
            } else {
                let rec = &app.recommendations[app.selected_index];
                if rec.risk != Risk::Danger {
                    app.selected_for_deletion.insert(app.selected_index);
                }
            }
        }
        KeyCode::Enter => {
            app.mode = AppMode::Details(app.selected_index);
        }
        KeyCode::Char('a') => {
            // Select all safe
            for (i, rec) in app.recommendations.iter().enumerate() {
                if rec.risk == Risk::Safe {
                    app.selected_for_deletion.insert(i);
                }
            }
        }
        KeyCode::Char('c') => {
            if !app.selected_for_deletion.is_empty() {
                app.confirm_target = app::ConfirmTarget::Recommendations;
                app.mode = AppMode::ConfirmDeletion;
            }
        }
        KeyCode::Char('?') => {
            app.mode = AppMode::Help;
        }
        _ => {}
    }
}

fn handle_details_input(app: &mut App, key: KeyCode, idx: usize) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.mode = AppMode::Results,
        KeyCode::Char('y') => {
            let rec = &app.recommendations[idx];
            if rec.risk != Risk::Danger {
                app.selected_for_deletion.insert(idx);
            }
            app.mode = AppMode::Results;
        }
        KeyCode::Char('n') => {
            app.selected_for_deletion.remove(&idx);
            app.mode = AppMode::Results;
        }
        _ => {}
    }
}

fn handle_confirm_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.confirmed_deletion = true;
            app.mode = AppMode::Quit;
        }
        KeyCode::Char('n') | KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::Results;
        }
        _ => {}
    }
}

fn handle_help_input(app: &mut App, key: KeyCode) {
    if matches!(key, KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter) {
        app.mode = AppMode::Menu;
    }
}

fn start_scan(app: &mut App) -> (ScanHandle, ScanReceiver) {
    app.mode = AppMode::Scanning;
    app.scan_complete = false;
    app.scan_cancelled = false;
    app.scan_frame = 0;

    let engine = AdvisorEngine {
        min_size: app.min_size,
        risk_limit: app.risk_limit.clone(),
        categories: app.categories.clone(),
        no_dev: !app.dev_mode,
        no_ai: false,
        older_than_days: None,
    };

    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        let recs = engine.scan_home();
        let _ = tx.send(recs);
    });

    (handle, rx)
}

fn start_disk_scan(_app: &mut App) -> (DiskHandle, DiskReceiver) {
    let home = std::env::var("HOME").unwrap_or_default();
    let home_path = std::path::PathBuf::from(&home);

    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        let mut folders = user_folders::scan_user_folders(&home_path);
        let home_summary = user_folders::scan_home_summary(&home_path);
        
        // Insert Home as first entry
        if let Some(ref home) = home_summary {
            folders.insert(0, home.clone());
        }
        
        let _ = tx.send((folders, home_summary));
    });

    (handle, rx)
}

fn execute_deletions(app: &App) {
    use std::process::Command;

    println!("\n🗑️  Executing deletions...\n");

    let mut indices: Vec<_> = app.selected_for_deletion.iter().copied().collect();
    indices.sort();

    for idx in indices {
        if let Some(rec) = app.recommendations.get(idx) {
            println!("  Deleting: {}", rec.path);
            println!("  Command: {}", rec.suggested_command);

            // Execute the command
            let output = Command::new("sh")
                .arg("-c")
                .arg(&rec.suggested_command)
                .output();

            match output {
                Ok(o) if o.status.success() => {
                    println!("  ✅ Deleted successfully\n");
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    println!("  ❌ Failed: {}\n", stderr);
                }
                Err(e) => {
                    println!("  ❌ Error: {}\n", e);
                }
            }
        }
    }

    println!("✨ Cleanup complete!");
}

fn execute_app_uninstalls(app: &App) {
    use std::process::Command;

    println!("\n📦 Uninstalling applications...\n");

    let mut indices: Vec<_> = app.apps_for_uninstall.iter().copied().collect();
    indices.sort();

    for idx in indices {
        if let Some(app_info) = app.apps.get(idx) {
            println!("  Uninstalling: {} ({})", app_info.name, crate::advisor::models::human_bytes(app_info.total_size));

            // Remove app bundle
            println!("    Removing: {}", app_info.bundle_path);
            let output = Command::new("rm")
                .arg("-rf")
                .arg(&app_info.bundle_path)
                .output();

            match output {
                Ok(o) if o.status.success() => {
                    println!("    ✅ App bundle removed");
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    println!("    ❌ Failed: {}", stderr);
                }
                Err(e) => {
                    println!("    ❌ Error: {}", e);
                }
            }

            // Remove support directories
            for dir in &app_info.support_dirs {
                println!("    Removing: {}", dir);
                let output = Command::new("rm")
                    .arg("-rf")
                    .arg(dir)
                    .output();

                match output {
                    Ok(o) if o.status.success() => {
                        println!("    ✅ Removed");
                    }
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        println!("    ❌ Failed: {}", stderr);
                    }
                    Err(e) => {
                        println!("    ❌ Error: {}", e);
                    }
                }
            }
            println!();
        }
    }

    println!("✨ Uninstall complete!");
}

fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    let (num_part, multiplier) = if s.ends_with("TB") || s.ends_with("T") {
        (
            s.trim_end_matches(|c: char| c.is_alphabetic()),
            1024u64 * 1024 * 1024 * 1024,
        )
    } else if s.ends_with("GB") || s.ends_with("G") {
        (
            s.trim_end_matches(|c: char| c.is_alphabetic()),
            1024u64 * 1024 * 1024,
        )
    } else if s.ends_with("MB") || s.ends_with("M") {
        (
            s.trim_end_matches(|c: char| c.is_alphabetic()),
            1024u64 * 1024,
        )
    } else if s.ends_with("KB") || s.ends_with("K") {
        (s.trim_end_matches(|c: char| c.is_alphabetic()), 1024u64)
    } else {
        (s.as_str(), 1u64)
    };
    num_part.trim().parse::<u64>().ok().map(|n| n * multiplier)
}

fn parse_risk(s: &str) -> Risk {
    match s.to_lowercase().as_str() {
        "safe" => Risk::Safe,
        "low" => Risk::Low,
        "medium" => Risk::Medium,
        "review" => Risk::Review,
        _ => Risk::Review,
    }
}
