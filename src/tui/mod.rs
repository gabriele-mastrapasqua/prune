pub mod app;
pub mod views;

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::advisor::scanner::AdvisorEngine;
use crate::advisor::models::{Category, Risk};
use app::{App, AppMode};

pub fn run(dev_mode: bool, min_size: Option<&str>, risk_limit: Option<&str>) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Parse options
    let min_size_bytes = min_size
        .and_then(parse_size)
        .unwrap_or(100 * 1024 * 1024); // default 100 MB

    let risk = risk_limit
        .map(parse_risk)
        .unwrap_or(Risk::Review);

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
    if !app.selected_for_deletion.is_empty() && app.confirmed_deletion {
        execute_deletions(&app);
    }

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            match app.mode {
                AppMode::Menu => views::menu::render(f, app),
                AppMode::Scanning => views::scan::render(f, app),
                AppMode::Results => views::results::render(f, app),
                AppMode::Details(idx) => views::details::render(f, app, idx),
                AppMode::ConfirmDeletion => views::confirm::render(f, app),
                AppMode::Help => views::help::render(f, app),
                AppMode::Quit => {}
            }
        })?;

        if matches!(app.mode, AppMode::Quit) {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.mode {
                AppMode::Menu => handle_menu_input(app, key.code),
                AppMode::Scanning => handle_scan_input(app, key.code),
                AppMode::Results => handle_results_input(app, key.code),
                AppMode::Details(idx) => handle_details_input(app, key.code, idx),
                AppMode::ConfirmDeletion => handle_confirm_input(app, key.code),
                AppMode::Help => handle_help_input(app, key.code),
                AppMode::Quit => {}
            }
        }

        // If scanning, check if done
        if matches!(app.mode, AppMode::Scanning) && app.scan_complete {
            app.mode = AppMode::Results;
        }
    }
}

fn handle_menu_input(app: &mut App, key: KeyCode) {
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
                    start_scan(app);
                }
                1 => {
                    // Scan without DEV
                    app.dev_mode = false;
                    start_scan(app);
                }
                2 => {
                    // Scan AI only
                    app.dev_mode = false;
                    app.categories = Some(vec![Category::AiModel]);
                    start_scan(app);
                }
                3 => {
                    // View history (TODO)
                }
                4 => {
                    // Settings (TODO)
                }
                5 => {
                    app.mode = AppMode::Help;
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn handle_scan_input(app: &mut App, key: KeyCode) {
    if matches!(key, KeyCode::Char('q') | KeyCode::Esc) {
        app.mode = AppMode::Menu;
    }
}

fn handle_results_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.mode = AppMode::Menu,
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

fn start_scan(app: &mut App) {
    app.mode = AppMode::Scanning;
    app.scan_complete = false;

    let engine = AdvisorEngine {
        min_size: app.min_size,
        risk_limit: app.risk_limit.clone(),
        categories: app.categories.clone(),
        no_dev: !app.dev_mode,
        no_ai: false,
        older_than_days: None,
    };

    // Run scan in background (simplified - runs synchronously for now)
    let recs = engine.scan_home();
    app.recommendations = recs;
    app.scan_complete = true;
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

fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    let (num_part, multiplier) = if s.ends_with("TB") || s.ends_with("T") {
        (s.trim_end_matches(|c: char| c.is_alphabetic()), 1024u64 * 1024 * 1024 * 1024)
    } else if s.ends_with("GB") || s.ends_with("G") {
        (s.trim_end_matches(|c: char| c.is_alphabetic()), 1024u64 * 1024 * 1024)
    } else if s.ends_with("MB") || s.ends_with("M") {
        (s.trim_end_matches(|c: char| c.is_alphabetic()), 1024u64 * 1024)
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
