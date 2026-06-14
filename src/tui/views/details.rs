use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::advisor::models::Risk;
use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App, idx: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // title
            Constraint::Min(15),   // content
            Constraint::Length(3), // footer
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("📋 Recommendation Details")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(title, chunks[0]);

    // Content
    if let Some(rec) = app.recommendations.get(idx) {
        let is_selected = app.selected_for_deletion.contains(&idx);

        let risk_style = match rec.risk {
            Risk::Safe => Style::default().fg(Color::Green),
            Risk::Low => Style::default().fg(Color::Yellow),
            Risk::Medium => Style::default().fg(Color::Rgb(255, 165, 0)),
            Risk::Review => Style::default().fg(Color::Red),
            Risk::Danger => Style::default().fg(Color::DarkGray),
        };

        let risk_text = match rec.risk {
            Risk::Safe => "✅ SAFE",
            Risk::Low => "🟡 LOW",
            Risk::Medium => "🟠 MEDIUM",
            Risk::Review => "🔴 REVIEW",
            Risk::Danger => "⛔ DANGER",
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Path: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(&rec.path),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Size: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(crate::advisor::models::human_bytes(rec.size), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Risk: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(risk_text, risk_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Category: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:?}", rec.category)),
            ]),
        ];

        if let Some(days) = rec.last_accessed_days {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Last accessed: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} days ago", days)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Reason:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(format!("  {}", rec.reason)));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Suggested command:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {}", rec.suggested_command), Style::default().fg(Color::Green)),
        ]));

        if is_selected {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "✓ Marked for deletion",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
        }

        let content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Details ")
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(content, chunks[1]);
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("y", Style::default().fg(Color::Green)),
        Span::raw(" mark for deletion  "),
        Span::styled("n", Style::default().fg(Color::Red)),
        Span::raw(" skip  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" back"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(footer, chunks[2]);
}
