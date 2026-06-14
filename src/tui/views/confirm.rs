use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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
    let title = Paragraph::new("⚠️  Confirm Deletion")
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(title, chunks[0]);

    // Content
    let selected_count = app.selected_for_deletion.len();
    let selected_size = app.selected_size();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("You are about to delete ", Style::default().fg(Color::White)),
            Span::styled(format!("{} items", selected_count), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" totaling ", Style::default().fg(Color::White)),
            Span::styled(crate::advisor::models::human_bytes(selected_size), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Items to delete:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];

    let mut indices: Vec<_> = app.selected_for_deletion.iter().copied().collect();
    indices.sort();

    for (i, &idx) in indices.iter().enumerate() {
        if i >= 10 {
            lines.push(Line::from(format!("  ... and {} more", selected_count - 10)));
            break;
        }
        if let Some(rec) = app.recommendations.get(idx) {
            let path_short = if rec.path.len() > 60 {
                format!("...{}", &rec.path[rec.path.len() - 57..])
            } else {
                rec.path.clone()
            };
            lines.push(Line::from(vec![
                Span::raw("  • "),
                Span::styled(path_short, Style::default().fg(Color::White)),
                Span::raw(" ("),
                Span::styled(crate::advisor::models::human_bytes(rec.size), Style::default().fg(Color::Cyan)),
                Span::raw(")"),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "This action cannot be undone!",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )));

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirmation ")
                .title_style(Style::default().fg(Color::Red)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(content, chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(" YES, delete", Style::default().fg(Color::Green)),
        Span::raw("  |  "),
        Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::styled(" NO, cancel", Style::default().fg(Color::Red)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(footer, chunks[2]);
}
