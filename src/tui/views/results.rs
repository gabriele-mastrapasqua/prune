use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::advisor::models::{Category, Risk};
use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(10),   // list
            Constraint::Length(3), // footer
        ])
        .split(f.size());

    // Header
    let total = app.total_recoverable();
    let selected = app.selected_count();
    let selected_size = app.selected_size();

    let header_text = Line::from(vec![
        Span::styled("📊 Scan Results", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" - "),
        Span::styled(format!("{}", crate::advisor::models::human_bytes(total)), Style::default().fg(Color::Green)),
        Span::raw(" recoverable"),
        Span::raw("  |  "),
        Span::styled(format!("{} selected", selected), Style::default().fg(Color::Yellow)),
        Span::raw(" ("),
        Span::styled(format!("{}", crate::advisor::models::human_bytes(selected_size)), Style::default().fg(Color::Yellow)),
        Span::raw(")"),
    ]);

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(header, chunks[0]);

    // Results list
    let items: Vec<ListItem> = app
        .recommendations
        .iter()
        .enumerate()
        .map(|(i, rec)| {
            let is_selected = app.selected_for_deletion.contains(&i);
            let is_current = i == app.selected_index;

            let checkbox = if is_selected {
                Span::styled("✓", Style::default().fg(Color::Green))
            } else {
                Span::styled("○", Style::default().fg(Color::DarkGray))
            };

            let risk_badge = match rec.risk {
                Risk::Safe => Span::styled("✅ SAFE", Style::default().fg(Color::Green)),
                Risk::Low => Span::styled("🟡 LOW", Style::default().fg(Color::Yellow)),
                Risk::Medium => Span::styled("🟠 MED", Style::default().fg(Color::Rgb(255, 165, 0))),
                Risk::Review => Span::styled("🔴 REV", Style::default().fg(Color::Red)),
                Risk::Danger => Span::styled("⛔ DNG", Style::default().fg(Color::DarkGray)),
            };

            let category_icon = match rec.category {
                Category::Cache => "🗑️",
                Category::Log => "📋",
                Category::Dev(_) => "🛠️",
                Category::AiModel => "🤖",
                Category::UserOldFiles => "📄",
                Category::Installer => "💿",
                Category::SystemTemp => "🗑️",
                Category::Duplicate => "📑",
                Category::Snapshot => "📸",
                Category::Unknown => "❓",
            };

            let size_str = crate::advisor::models::human_bytes(rec.size);

            let style = if is_current {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let path_short = if rec.path.len() > 50 {
                format!("...{}", &rec.path[rec.path.len() - 47..])
            } else {
                rec.path.clone()
            };

            ListItem::new(Line::from(vec![
                checkbox,
                Span::raw(" "),
                Span::styled(format!("{} ", category_icon), style),
                Span::styled(format!("{:>8}", size_str), Style::default().fg(Color::Cyan)),
                Span::raw(" │ "),
                risk_badge,
                Span::raw(" │ "),
                Span::styled(path_short, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Recommendations ({}) ", app.recommendations.len()))
            .title_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
        Span::raw(" nav  "),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::raw(" toggle  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" details  "),
        Span::styled("a", Style::default().fg(Color::Yellow)),
        Span::raw(" all safe  "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(" cleanup  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" back"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(footer, chunks[2]);
}
