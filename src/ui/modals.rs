use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::layout::centered_rect;
use crate::app::App;

/// Draw the help modal overlay
pub fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from("LazyCelery - Keyboard Shortcuts"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  Tab       - Switch between tabs"),
        Line::from("  ↑/k       - Move up"),
        Line::from("  ↓/j       - Move down"),
        Line::from("  Enter/d   - View details (in Tasks tab)"),
        Line::from("  Esc       - Go back"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  /         - Search"),
        Line::from("  p         - Purge queue (in Queues tab)"),
        Line::from("  r         - Retry task (in Tasks tab)"),
        Line::from("  x         - Revoke task (in Tasks tab)"),
        Line::from(""),
        Line::from("General:"),
        Line::from("  ?         - Toggle this help"),
        Line::from("  q         - Quit application"),
        Line::from(""),
        Line::from("Press any key to close this help..."),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help, area);
}

/// Draw the confirmation dialog modal
pub fn draw_confirmation_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let confirmation_text = vec![
        Line::from(""),
        Line::from(app.confirmation_message.clone()),
        Line::from(""),
        Line::from("Press [y/Enter] to confirm or [n/Esc] to cancel"),
    ];

    let confirmation = Paragraph::new(confirmation_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirmation ")
                .style(Style::default().bg(Color::Black).fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(confirmation, area);
}

/// Draw the detailed task information modal
pub fn draw_task_details_modal(f: &mut Frame, app: &App) {
    if let Some(task) = &app.selected_task_details {
        let popup_area = centered_rect(80, 70, f.area());

        // Clear background
        f.render_widget(Clear, popup_area);

        // Draw modal background
        f.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Task Details ")
                .style(Style::default().bg(Color::Black)),
            popup_area,
        );

        let inner_area = Layout::default()
            .margin(1)
            .constraints([Constraint::Percentage(100)])
            .split(popup_area)[0];

        // Create task details content
        let details_lines = build_task_details_content(task);

        let paragraph = Paragraph::new(details_lines)
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        f.render_widget(paragraph, inner_area);
    }
}

/// Build the content lines for task details modal
fn build_task_details_content(task: &crate::models::Task) -> Vec<Line> {
    let mut details_lines = vec![
        Line::from(vec![
            Span::styled(
                "ID: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(task.id.clone()),
        ]),
        Line::from(vec![
            Span::styled(
                "Name: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(task.name.clone()),
        ]),
        Line::from(vec![
            Span::styled(
                "Status: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:?}", task.status),
                Style::default().fg(get_status_color(&task.status)),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "Worker: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(task.worker.as_deref().unwrap_or("Unknown").to_string()),
        ]),
        Line::from(vec![
            Span::styled(
                "Queue: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("default".to_string()),
        ]),
        Line::from(vec![
            Span::styled(
                "Timestamp: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(task.timestamp.to_string()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Arguments: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(task.args.as_str()),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Keyword Arguments: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(task.kwargs.as_str()),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Result: ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(task.result.as_deref().unwrap_or("None")),
    ];

    // Add traceback if available and task failed
    if task.status == crate::models::TaskStatus::Failure {
        if let Some(traceback) = &task.traceback {
            details_lines.push(Line::from(""));
            details_lines.push(Line::from(vec![Span::styled(
                "Traceback: ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )]));
            // Split traceback into lines and add them
            for line in traceback.lines() {
                details_lines.push(Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Red),
                )));
            }
        }
    }

    // Add footer
    details_lines.push(Line::from(""));
    details_lines.push(Line::from(vec![Span::styled(
        "Press any key to close",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::ITALIC),
    )]));

    details_lines
}

/// Get the appropriate color for a task status
fn get_status_color(status: &crate::models::TaskStatus) -> Color {
    match status {
        crate::models::TaskStatus::Success => Color::Green,
        crate::models::TaskStatus::Failure => Color::Red,
        crate::models::TaskStatus::Retry => Color::Yellow,
        crate::models::TaskStatus::Pending => Color::Blue,
        crate::models::TaskStatus::Revoked => Color::Magenta,
        _ => Color::White,
    }
}
