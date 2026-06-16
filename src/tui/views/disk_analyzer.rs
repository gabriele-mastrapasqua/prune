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
    let total_size: u64 = app.folder_summaries.iter().map(|f| f.total_size).sum();
    let total_files: u64 = app.folder_summaries.iter().map(|f| f.file_count).sum();

    let header_text = Line::from(vec![
        Span::styled(
            "💾 Disk Analyzer",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled(
            format!("{} folders", app.folder_summaries.len()),
            Style::default().fg(Color::White),
        ),
        Span::raw(" ("),
        Span::styled(
            human_bytes(total_size),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(", "),
        Span::styled(
            format!("{} files", total_files),
            Style::default().fg(Color::White),
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

    // Folder list or scanning indicator
    if !app.disk_scan_complete {
        let spinner_frames = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame_idx = app.scan_frame % spinner_frames.len();
        
        // Animated progress bar (bounce between 10% and 90%)
        let cycle = app.scan_frame % 20;
        let progress_pct = if cycle < 10 {
            10 + cycle * 8
        } else {
            10 + (19 - cycle) * 8
        };
        
        let progress_width = 40;
        let filled = (progress_width * progress_pct / 100) as usize;
        let progress_bar = format!(
            "{}{} {}%",
            "█".repeat(filled),
            "░".repeat(progress_width - filled),
            progress_pct
        );
        
        let scanning_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!("{} Scanning user folders...", spinner_frames[frame_idx]),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                &progress_bar,
                Style::default().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Analyzing file sizes and access times",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" User Folders ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(scanning_msg, left_chunks[1]);
    } else if app.folder_summaries.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "No user folders found",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" User Folders ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(empty_msg, left_chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .folder_summaries
            .iter()
            .enumerate()
            .map(|(i, folder)| {
            let is_current = i == app.disk_selected_index;
            let pct = if total_size > 0 {
                (folder.total_size as f64 / total_size as f64 * 100.0) as u64
            } else {
                0
            };

            let bar_width = 20;
            let filled = (bar_width as u64 * pct / 100).max(1) as usize;
            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_width - filled));

            let style = if is_current {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let name_display = if folder.name.len() > 15 {
                format!("{}...", &folder.name[..12])
            } else {
                folder.name.clone()
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<16}", name_display), style),
                Span::styled(bar, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(
                    format!("{:>8}", human_bytes(folder.total_size)),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("({}%)", pct),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{}f", folder.file_count),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" User Folders ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(list, left_chunks[1]);
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
        Span::raw(" nav  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" details  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" sidebar  "),
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
        render_sidebar(f, app, main_chunks[1]);
    }
}

fn render_sidebar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Title
    let sidebar_title = Paragraph::new("📊 Folder Details")
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
    f.render_widget(sidebar_title, chunks[0]);

    if let Some(folder) = app.folder_summaries.get(app.disk_selected_index) {
        // Stats
        let stats = vec![
            Line::from(vec![
                Span::styled("Name:    ", Style::default().fg(Color::Yellow)),
                Span::raw(&folder.name),
            ]),
            Line::from(vec![
                Span::styled("Path:    ", Style::default().fg(Color::Yellow)),
                Span::raw(folder.path.display().to_string()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Total:   ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    human_bytes(folder.total_size),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Files:   ", Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}", folder.file_count)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Old files (90d+):  ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}", folder.old_files_90d),
                    Style::default().fg(if folder.old_files_90d > 100 {
                        Color::Red
                    } else {
                        Color::White
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("Old files (180d+): ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}", folder.old_files_180d),
                    Style::default().fg(if folder.old_files_180d > 50 {
                        Color::Red
                    } else {
                        Color::White
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("Old files (365d+): ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}", folder.old_files_365d),
                    Style::default().fg(if folder.old_files_365d > 20 {
                        Color::Red
                    } else {
                        Color::White
                    }),
                ),
            ]),
        ];

        let stats_block = Paragraph::new(stats)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Statistics ")
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(stats_block, chunks[1]);

        // Top subfolders
        let subfolder_items: Vec<ListItem> = folder
            .subfolder_sizes
            .iter()
            .take(8)
            .map(|(name, size)| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{:<20}", name), Style::default().fg(Color::White)),
                    Span::styled(
                        format!("{:>8}", human_bytes(*size)),
                        Style::default().fg(Color::Cyan),
                    ),
                ]))
            })
            .collect();

        let subfolder_list = List::new(subfolder_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Top Subfolders ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(subfolder_list, chunks[2]);

        // Top files
        let file_items: Vec<ListItem> = folder
            .top_files
            .iter()
            .take(8)
            .map(|f| {
                let days_ago = f.last_accessed_days.unwrap_or(0);
                let age_color = if days_ago > 365 {
                    Color::Red
                } else if days_ago > 180 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:<22}", if f.name.len() > 22 { &f.name[..19] } else { &f.name }),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("{:>7}", human_bytes(f.size)),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:>4}d", days_ago),
                        Style::default().fg(age_color),
                    ),
                ]))
            })
            .collect();

        let file_list = List::new(file_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Largest Files ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(file_list, chunks[3]);
    }
}
