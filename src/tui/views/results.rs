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
    let total = app.total_recoverable();
    let selected = app.selected_count();
    let selected_size = app.selected_size();

    let header_text = Line::from(vec![
        Span::styled(
            "📊 Scan Results",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled(
            crate::advisor::models::human_bytes(total),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" recoverable"),
        Span::raw("  |  "),
        Span::styled(
            format!("{} selected", selected),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" ("),
        Span::styled(
            crate::advisor::models::human_bytes(selected_size),
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
                Risk::Medium => {
                    Span::styled("🟠 MED", Style::default().fg(Color::Rgb(255, 165, 0)))
                }
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
                Category::Application => "📦",
                Category::AutoUpdate => "🔄",
                Category::DevTool => "🧠",
                Category::Unknown => "❓",
            };

            let size_str = crate::advisor::models::human_bytes(rec.size);

            let style = if is_current {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let path_short = smart_shorten_path(&rec.path, 40);

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
    f.render_widget(list, left_chunks[1]);

    // Footer
    let mut footer_spans = vec![
        Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
        Span::raw(" nav  "),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::raw(" toggle  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" sidebar  "),
        Span::styled("a", Style::default().fg(Color::Yellow)),
        Span::raw(" all safe  "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(" cleanup  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" back"),
    ];
    if !show_sidebar {
        footer_spans = vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" nav  "),
            Span::styled("Space", Style::default().fg(Color::Yellow)),
            Span::raw(" toggle  "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(" sidebar  "),
            Span::styled("a", Style::default().fg(Color::Yellow)),
            Span::raw(" all safe  "),
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw(" cleanup  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" back"),
        ];
    }

    let footer = Paragraph::new(Line::from(footer_spans))
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
                Constraint::Length(3), // sidebar title
                Constraint::Min(10),   // sidebar content
            ])
            .split(main_chunks[1]);

        let sidebar_title = Paragraph::new("📋 Details")
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

        if let Some(rec) = app.recommendations.get(app.selected_index) {
            let is_selected = app.selected_for_deletion.contains(&app.selected_index);

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

            let category_text = match &rec.category {
                Category::Cache => "🗑️ Cache".to_string(),
                Category::Log => "📋 Log".to_string(),
                Category::Dev(kind) => format!("🛠️ Dev::{:?}", kind),
                Category::AiModel => "🤖 AI Model".to_string(),
                Category::UserOldFiles => "📄 User Files".to_string(),
                Category::Installer => "💿 Installer".to_string(),
                Category::SystemTemp => "🗑️ System Temp".to_string(),
                Category::Duplicate => "📑 Duplicate".to_string(),
                Category::Snapshot => "📸 Snapshot".to_string(),
                Category::Application => "📦 Application".to_string(),
                Category::AutoUpdate => "🔄 Auto-Update".to_string(),
                Category::DevTool => "🧠 Dev Tool".to_string(),
                Category::Unknown => "❓ Unknown".to_string(),
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::styled(
                        "Path:\n",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&rec.path, Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "Size:  ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        crate::advisor::models::human_bytes(rec.size),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "Risk:  ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(risk_text, risk_style),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        "Type:  ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(category_text),
                ]),
            ];

            if let Some(days) = rec.last_accessed_days {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled(
                        "Last used:  ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("{} days ago", days)),
                ]));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Reason:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(format!("  {}", rec.reason)));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "Command:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(vec![Span::styled(
                format!("  {}", rec.suggested_command),
                Style::default().fg(Color::Green),
            )]));

            if is_selected {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "✓ Marked for deletion",
                    Style::default()
                        .fg(Color::Green)
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

fn smart_shorten_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    let home_replacements: &[(&str, &str)] = &[
        ("/Users/", "~/"),
        ("/home/", "~/"),
    ];

    let mut shortened = path.to_string();
    for (from, to) in home_replacements {
        if let Some(pos) = shortened.find(from) {
            let after = &shortened[pos + from.len()..];
            if let Some(slash_pos) = after.find('/') {
                shortened = format!("{}{}/{}", to, &after[..slash_pos], &after[slash_pos + 1..]);
            }
        }
    }

    if shortened.len() <= max_len {
        return format!("...{}", &shortened[shortened.len().saturating_sub(max_len - 3)..]);
    }

    let parts: Vec<&str> = shortened.split('/').collect();
    if parts.len() <= 2 {
        return format!("...{}", &shortened[shortened.len().saturating_sub(max_len - 3)..]);
    }

    let last = parts[parts.len() - 1];
    let second_last = parts[parts.len() - 2];
    let suffix = format!("{}/{}", second_last, last);

    if suffix.len() + 3 <= max_len {
        return format!(".../{}", suffix);
    }

    if last.len() + 3 <= max_len {
        return format!(".../{}", last);
    }

    format!("...{}", &last[last.len().saturating_sub(max_len - 3)..])
}
