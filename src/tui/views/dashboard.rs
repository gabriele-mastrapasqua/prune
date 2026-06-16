use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

use crate::advisor::models::{human_bytes, Category, Risk};
use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(20),   // content
            Constraint::Length(3), // footer
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("📊 Scan Dashboard")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(title, chunks[0]);

    // Content: 2x2 grid
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(50), // top row
            Constraint::Percentage(50), // bottom row
        ])
        .split(chunks[1]);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(content_chunks[0]);

    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(content_chunks[1]);

    // Panel 1: Disk Overview
    render_disk_overview(f, top_row[0]);

    // Panel 2: Risk Summary
    render_risk_summary(f, app, top_row[1]);

    // Panel 3: Top 10 Heaviest
    render_top_heaviest(f, app, bottom_row[0]);

    // Panel 4: Category Breakdown
    render_category_breakdown(f, app, bottom_row[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" view results  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" back to menu"),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, chunks[2]);
}

fn render_disk_overview(f: &mut Frame, area: ratatui::layout::Rect) {
    let mut disks = sysinfo::Disks::new();
    disks.refresh_list();

    let home = std::env::var("HOME").unwrap_or_default();
    let home_path = std::path::Path::new(&home);

    let mut total_space = 0u64;
    let mut available_space = 0u64;

    for disk in disks.list() {
        let mount_point = disk.mount_point();
        if home_path.starts_with(mount_point) {
            total_space = disk.total_space();
            available_space = disk.available_space();
            break;
        }
    }

    let used_space = total_space.saturating_sub(available_space);
    let usage_pct = if total_space > 0 {
        (used_space as f64 / total_space as f64 * 100.0) as u64
    } else {
        0
    };

    let bar_width = area.width.saturating_sub(12) as usize;
    let filled = (bar_width as u64 * usage_pct / 100) as usize;
    let empty = bar_width.saturating_sub(filled);

    let bar = format!(
        "{}{}  {}%",
        "█".repeat(filled),
        "░".repeat(empty),
        usage_pct
    );

    let bar_color = if usage_pct > 90 {
        Color::Red
    } else if usage_pct > 70 {
        Color::Yellow
    } else {
        Color::Green
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Total:      ", Style::default().fg(Color::Yellow)),
            Span::styled(human_bytes(total_space), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Used:       ", Style::default().fg(Color::Yellow)),
            Span::styled(human_bytes(used_space), Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::styled("Available:  ", Style::default().fg(Color::Yellow)),
            Span::styled(human_bytes(available_space), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(Span::styled(bar, Style::default().fg(bar_color))),
    ];

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 💾 Disk Usage ")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(panel, area);
}

fn render_risk_summary(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut risk_counts: HashMap<String, (usize, u64)> = HashMap::new();

    for rec in &app.recommendations {
        let key = match rec.risk {
            Risk::Safe => "✅ Safe",
            Risk::Low => "🟡 Low",
            Risk::Medium => "🟠 Medium",
            Risk::Review => "🔴 Review",
            Risk::Danger => "⛔ Danger",
        };
        let entry = risk_counts.entry(key.to_string()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += rec.size;
    }

    let total = app.total_recoverable();
    let mut lines = vec![Line::from("")];

    let risk_order = ["✅ Safe", "🟡 Low", "🟠 Medium", "🔴 Review", "⛔ Danger"];
    for risk_name in &risk_order {
        if let Some((count, size)) = risk_counts.get(*risk_name) {
            let pct = if total > 0 {
                (*size as f64 / total as f64 * 100.0) as u64
            } else {
                0
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}  ", risk_name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:>3} items", count),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:>8}", human_bytes(*size)),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  "),
                Span::styled(format!("({}%)", pct), Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Total:  ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{} items", app.recommendations.len()),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled(
            human_bytes(total),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🎯 Risk Summary ")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(panel, area);
}

fn render_top_heaviest(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let top_items: Vec<_> = app.recommendations.iter().take(10).collect();

    if top_items.is_empty() {
        let panel = Paragraph::new("  No items found")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🔥 Top 10 Heaviest ")
                    .title_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(panel, area);
        return;
    }

    let max_size = top_items[0].size;
    let bar_max_width = area.width.saturating_sub(30) as usize;

    let mut lines = vec![Line::from("")];

    for (i, rec) in top_items.iter().enumerate() {
        let bar_width = if max_size > 0 {
            (rec.size as f64 / max_size as f64 * bar_max_width as f64) as usize
        } else {
            0
        };
        let bar_width = bar_width.max(1);

        let path_short = if rec.path.len() > 25 {
            format!("...{}", &rec.path[rec.path.len() - 22..])
        } else {
            rec.path.clone()
        };

        let bar_color = match rec.risk {
            Risk::Safe => Color::Green,
            Risk::Low => Color::Yellow,
            Risk::Medium => Color::Rgb(255, 165, 0),
            Risk::Review => Color::Red,
            Risk::Danger => Color::DarkGray,
        };

        lines.push(Line::from(vec![
            Span::styled(format!(" {:>2}. ", i + 1), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:>8}", human_bytes(rec.size)), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled("█".repeat(bar_width), Style::default().fg(bar_color)),
            Span::raw(" "),
            Span::raw(path_short),
        ]));
    }

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🔥 Top 10 Heaviest ")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(panel, area);
}

fn render_category_breakdown(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut cat_sizes: Vec<(String, u64)> = Vec::new();
    let mut cat_map: HashMap<String, u64> = HashMap::new();

    for rec in &app.recommendations {
        let key = match &rec.category {
            Category::Cache => "🗑️ Cache".to_string(),
            Category::Log => "📋 Logs".to_string(),
            Category::Dev(kind) => format!("🛠️ Dev::{:?}", kind),
            Category::AiModel => "🤖 AI Models".to_string(),
            Category::UserOldFiles => "📄 Old Files".to_string(),
            Category::Installer => "💿 Installers".to_string(),
            Category::SystemTemp => "🗑️ Sys Temp".to_string(),
            Category::Duplicate => "📑 Duplicates".to_string(),
            Category::Snapshot => "📸 Snapshots".to_string(),
            Category::Application => "📦 Apps".to_string(),
            Category::AutoUpdate => "🔄 Updates".to_string(),
            Category::DevTool => "🧠 Dev Tools".to_string(),
            Category::Unknown => "❓ Unknown".to_string(),
        };
        *cat_map.entry(key).or_insert(0) += rec.size;
    }

    for (name, size) in cat_map {
        cat_sizes.push((name, size));
    }
    cat_sizes.sort_by_key(|b| std::cmp::Reverse(b.1));

    if cat_sizes.is_empty() {
        let panel = Paragraph::new("  No categories found")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 📂 Categories ")
                    .title_style(Style::default().fg(Color::Cyan)),
            );
        f.render_widget(panel, area);
        return;
    }

    let total: u64 = cat_sizes.iter().map(|(_, s)| s).sum();
    let bar_max_width = area.width.saturating_sub(30) as usize;

    let mut lines = vec![Line::from("")];

    for (name, size) in &cat_sizes {
        let pct = if total > 0 {
            (*size as f64 / total as f64 * 100.0) as u64
        } else {
            0
        };
        let bar_width = (bar_max_width as u64 * pct / 100).max(1) as usize;

        lines.push(Line::from(vec![
            Span::styled(format!("  {:<14}", name), Style::default().fg(Color::White)),
            Span::styled("█".repeat(bar_width), Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(format!("{:>8}", human_bytes(*size)), Style::default().fg(Color::Cyan)),
            Span::styled(format!(" ({}%)", pct), Style::default().fg(Color::DarkGray)),
        ]));
    }

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 📂 Category Breakdown ")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(panel, area);
}
