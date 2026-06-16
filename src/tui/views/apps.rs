use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::advisor::models::human_bytes;
use crate::tui::app::App;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.size();
    let show_sidebar = app.show_sidebar && area.width >= 100;

    let main_chunks = if show_sidebar {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(10),   // list
            Constraint::Length(3), // footer
        ])
        .split(main_chunks[0]);

    // Header
    let uninstall_count = app.apps_for_uninstall.len();
    let uninstall_size = app.apps_uninstall_size();

    let header_text = Line::from(vec![
        Span::styled(
            "📦 App Manager",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled(
            format!("{} apps", app.apps.len()),
            Style::default().fg(Color::White),
        ),
        Span::raw(" ("),
        Span::styled(
            human_bytes(app.apps_total_size()),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(")  |  "),
        Span::styled(
            format!("{} marked", uninstall_count),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" ("),
        Span::styled(
            human_bytes(uninstall_size),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(")"),
    ]);

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(header, left_chunks[0]);

    // App list
    let items: Vec<ListItem> = app
        .apps
        .iter()
        .enumerate()
        .map(|(i, app_info)| {
            let is_marked = app.apps_for_uninstall.contains(&i);
            let is_current = i == app.app_selected_index;

            let checkbox = if is_marked {
                Span::styled("✓", Style::default().fg(Color::Green))
            } else {
                Span::styled("○", Style::default().fg(Color::DarkGray))
            };

            let size_str = human_bytes(app_info.total_size);

            let last_used_style = match app_info.last_used_days {
                Some(d) if d > 180 => Style::default().fg(Color::Red),
                Some(d) if d > 90 => Style::default().fg(Color::Yellow),
                Some(d) if d > 30 => Style::default().fg(Color::White),
                _ => Style::default().fg(Color::Green),
            };

            let last_used_text = match app_info.last_used_days {
                Some(d) if d > 365 => format!("{}y ago", d / 365),
                Some(d) if d > 30 => format!("{}d ago", d),
                Some(d) => format!("{}d ago", d),
                None => "unknown".to_string(),
            };

            let version_text = app_info
                .version
                .as_deref()
                .unwrap_or("");

            let style = if is_current {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let name_display = if app_info.name.len() > 25 {
                format!("{}...", &app_info.name[..22])
            } else {
                app_info.name.clone()
            };

            ListItem::new(Line::from(vec![
                checkbox,
                Span::raw(" "),
                Span::styled(format!("{:<26}", name_display), style),
                Span::styled(format!("{:>8}", size_str), Style::default().fg(Color::Cyan)),
                Span::raw(" │ "),
                Span::styled(format!("{:>7}", last_used_text), last_used_style),
                Span::raw(" │ "),
                Span::styled(
                    format!("{:>8}", version_text),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Installed Applications ({}) ", app.apps.len()))
            .title_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, left_chunks[1]);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
        Span::raw(" nav  "),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::raw(" select  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" sidebar  "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(" confirm  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" back"),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, left_chunks[2]);

    // Sidebar
    if show_sidebar {
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
            ])
            .split(main_chunks[1]);

        let sidebar_title = Paragraph::new("📦 App Details")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        f.render_widget(sidebar_title, sidebar_chunks[0]);

        if let Some(app_info) = app.apps.get(app.app_selected_index) {
            let is_marked = app.apps_for_uninstall.contains(&app.app_selected_index);

            let last_used_text = match app_info.last_used_days {
                Some(d) if d > 365 => format!("{} years ago", d / 365),
                Some(d) if d > 60 => format!("{} months ago", d / 30),
                Some(d) => format!("{} days ago", d),
                None => "Unknown".to_string(),
            };

            let last_used_style = match app_info.last_used_days {
                Some(d) if d > 180 => Style::default().fg(Color::Red),
                Some(d) if d > 90 => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::Green),
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::styled(
                        "Name:\n",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&app_info.name, Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Bundle:    ", Style::default().fg(Color::Yellow)),
                    Span::raw(&app_info.bundle_id),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Version:   ", Style::default().fg(Color::Yellow)),
                    Span::raw(app_info.version.as_deref().unwrap_or("N/A")),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Last used: ", Style::default().fg(Color::Yellow)),
                    Span::styled(&last_used_text, last_used_style),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "App size:      ",
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        human_bytes(app_info.app_size),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Support data:  ",
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        human_bytes(app_info.support_size),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Cache data:    ",
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        human_bytes(app_info.cache_size),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "Total:         ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        human_bytes(app_info.total_size),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
            ];

            if !app_info.support_dirs.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Related files:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                for dir in &app_info.support_dirs {
                    let short = if dir.len() > 50 {
                        format!("...{}", &dir[dir.len() - 47..])
                    } else {
                        dir.clone()
                    };
                    lines.push(Line::from(format!("  {}", short)));
                }
            }

            if is_marked {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "✓ Marked for uninstall",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
            }

            if app_info.last_used_days.map(|d| d > 90).unwrap_or(false) {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "⚠ Not used in 90+ days",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
            }

            let sidebar_content = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Info ")
                        .title_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(sidebar_content, sidebar_chunks[1]);
        }
    }
}
