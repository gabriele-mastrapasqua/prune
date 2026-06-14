use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn render(f: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(15),   // content
            Constraint::Length(3), // footer
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("❓ Help")
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

    // Content
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Navigation:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  ↑/k  Move up"),
        Line::from("  ↓/j  Move down"),
        Line::from("  Enter  Select/Confirm"),
        Line::from("  q/Esc  Back/Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Results View:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Space  Toggle selection for deletion"),
        Line::from("  a      Select all SAFE items"),
        Line::from("  c      Cleanup selected items"),
        Line::from("  Enter  View details"),
        Line::from(""),
        Line::from(Span::styled(
            "Details View:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  y  Mark for deletion"),
        Line::from("  n  Skip"),
        Line::from(""),
        Line::from(Span::styled(
            "Risk Levels:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  ✅ SAFE    ", Style::default().fg(Color::Green)),
            Span::raw("No risk, can delete freely"),
        ]),
        Line::from(vec![
            Span::styled("  🟡 LOW     ", Style::default().fg(Color::Yellow)),
            Span::raw("Low risk, usually safe"),
        ]),
        Line::from(vec![
            Span::styled(
                "  🟠 MEDIUM  ",
                Style::default().fg(Color::Rgb(255, 165, 0)),
            ),
            Span::raw("Review before deleting"),
        ]),
        Line::from(vec![
            Span::styled("  🔴 REVIEW  ", Style::default().fg(Color::Red)),
            Span::raw("Manual review required"),
        ]),
        Line::from(vec![
            Span::styled("  ⛔ DANGER  ", Style::default().fg(Color::DarkGray)),
            Span::raw("Never auto-delete"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "About:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  prune v0.1.0 - Disk Cleanup Advisor for macOS"),
        Line::from("  Fork of dust with intelligent cleanup recommendations"),
    ];

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Keyboard Shortcuts ")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(content, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" or "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" to go back"),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, chunks[2]);
}
