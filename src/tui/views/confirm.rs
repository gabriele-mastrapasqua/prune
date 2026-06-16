use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{App, ConfirmTarget};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(10),   // content
            Constraint::Length(3), // footer
        ])
        .split(f.size());

    // Title
    let (title_text, title_color) = match app.confirm_target {
        ConfirmTarget::Recommendations => ("⚠️  Confirm Deletion", Color::Red),
        ConfirmTarget::Apps => ("⚠️  Confirm Uninstall", Color::Red),
    };

    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(title_color)
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
    let (count, size, items_lines) = match app.confirm_target {
        ConfirmTarget::Recommendations => {
            let count = app.selected_for_deletion.len();
            let size = app.selected_size();
            let mut lines = Vec::new();

            let mut indices: Vec<_> = app.selected_for_deletion.iter().copied().collect();
            indices.sort();

            for (i, &idx) in indices.iter().enumerate() {
                if i >= 10 {
                    lines.push(Line::from(format!(
                        "  ... and {} more",
                        count - 10
                    )));
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
                        Span::styled(
                            crate::advisor::models::human_bytes(rec.size),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::raw(")"),
                    ]));
                }
            }
            (count, size, lines)
        }
        ConfirmTarget::Apps => {
            let count = app.apps_for_uninstall.len();
            let size = app.apps_uninstall_size();
            let mut lines = Vec::new();

            let mut indices: Vec<_> = app.apps_for_uninstall.iter().copied().collect();
            indices.sort();

            for (i, &idx) in indices.iter().enumerate() {
                if i >= 10 {
                    lines.push(Line::from(format!(
                        "  ... and {} more",
                        count - 10
                    )));
                    break;
                }
                if let Some(app_info) = app.apps.get(idx) {
                    lines.push(Line::from(vec![
                        Span::raw("  • "),
                        Span::styled(&app_info.name, Style::default().fg(Color::White)),
                        Span::raw(" ("),
                        Span::styled(
                            crate::advisor::models::human_bytes(app_info.total_size),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::raw(")"),
                    ]));
                }
            }
            (count, size, lines)
        }
    };

    let mut content_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "You are about to ",
                Style::default().fg(Color::White),
            ),
            Span::styled(
                match app.confirm_target {
                    ConfirmTarget::Recommendations => "delete ",
                    ConfirmTarget::Apps => "uninstall ",
                },
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("{} items", count),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" totaling ", Style::default().fg(Color::White)),
            Span::styled(
                crate::advisor::models::human_bytes(size),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            match app.confirm_target {
                ConfirmTarget::Recommendations => "Items to delete:",
                ConfirmTarget::Apps => "Apps to uninstall:",
            },
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    content_lines.extend(items_lines);

    content_lines.push(Line::from(""));
    content_lines.push(Line::from(Span::styled(
        "This action cannot be undone!",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )));

    let content = Paragraph::new(content_lines)
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
        Span::styled(
            "y",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            match app.confirm_target {
                ConfirmTarget::Recommendations => " YES, delete",
                ConfirmTarget::Apps => " YES, uninstall",
            },
            Style::default().fg(Color::Green),
        ),
        Span::raw("  |  "),
        Span::styled(
            "n",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" NO, cancel", Style::default().fg(Color::Red)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, chunks[2]);
}
