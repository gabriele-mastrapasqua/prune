use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // title
            Constraint::Min(10),   // content
            Constraint::Length(3), // footer
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("🔍 Scanning...")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(title, chunks[0]);

    // Content
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // progress bar
            Constraint::Length(3),  // status
            Constraint::Min(0),    // info
        ])
        .split(chunks[1]);

    // Progress bar (simulated - in real implementation would track actual progress)
    let progress = if app.scan_complete { 100 } else { 50 };
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .percent(progress);
    f.render_widget(gauge, content_chunks[0]);

    // Status
    let status_text = if app.scan_complete {
        format!(
            "✅ Scan complete! Found {} recommendations",
            app.recommendations.len()
        )
    } else {
        "Scanning home directory...".to_string()
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    f.render_widget(status, content_chunks[1]);

    // Info
    let info_text = vec![
        Line::from(vec![
            Span::styled("DEV mode: ", Style::default().fg(Color::Yellow)),
            Span::raw(if app.dev_mode { "ON" } else { "OFF" }),
        ]),
        Line::from(vec![
            Span::styled("Min size: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", crate::advisor::models::human_bytes(app.min_size))),
        ]),
        Line::from(vec![
            Span::styled("Risk limit: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:?}", app.risk_limit)),
        ]),
    ];

    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title(" Scan Options "))
        .alignment(Alignment::Left);
    f.render_widget(info, content_chunks[2]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" cancel"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(footer, chunks[2]);
}
