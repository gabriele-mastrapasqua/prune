use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::tui::app::App;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

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

    // Title with animated spinner
    let frame_idx = app.scan_frame % SPINNER_FRAMES.len();
    let spinner = SPINNER_FRAMES[frame_idx];

    let title_text = if app.scan_complete {
        "✅ Scan complete!"
    } else {
        "Scanning..."
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{}  {}", spinner, title_text),
            Style::default()
                .fg(if app.scan_complete {
                    Color::Green
                } else {
                    Color::Yellow
                })
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(title, chunks[0]);

    // Content
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // progress bar
            Constraint::Length(3), // status
            Constraint::Min(0),    // info
        ])
        .split(chunks[1]);

    // Progress bar — bouncing indeterminate animation while scanning, 100% when done
    let (progress, label) = if app.scan_complete {
        (100, "Done!".to_string())
    } else {
        // Bounce between 10% and 90% using a sine-like cycle
        let cycle = app.scan_frame % 20;
        let pct = if cycle < 10 {
            10 + cycle * 8
        } else {
            10 + (19 - cycle) * 8
        };
        (pct as u16, format!("{} Scanning...", SPINNER_FRAMES[app.scan_frame % SPINNER_FRAMES.len()]))
    };
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(
            Style::default()
                .fg(if app.scan_complete {
                    Color::Green
                } else {
                    Color::Cyan
                })
                .bg(Color::DarkGray),
        )
        .percent(progress)
        .label(label);
    f.render_widget(gauge, content_chunks[0]);

    // Status
    let status_text = if app.scan_complete {
        format!(
            "✅ Found {} recommendations",
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
            Span::raw(crate::advisor::models::human_bytes(app.min_size)),
        ]),
        Line::from(vec![
            Span::styled("Risk limit: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:?}", app.risk_limit)),
        ]),
    ];

    let info = Paragraph::new(info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Scan Options "),
        )
        .alignment(Alignment::Left);
    f.render_widget(info, content_chunks[2]);

    // Footer
    let footer_text = if app.scan_complete {
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" view results  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" back to menu"),
        ])
    } else {
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel scan"),
        ])
    };

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(footer, chunks[2]);
}
