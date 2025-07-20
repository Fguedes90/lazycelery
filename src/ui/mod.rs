pub mod events;
pub mod widgets;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::app::{App, Tab};
use crate::ui::widgets::{QueueWidget, TaskWidget, WorkerWidget};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    // Draw header with tabs
    draw_header(f, app, chunks[0]);

    // Draw main content based on selected tab
    match app.selected_tab {
        Tab::Workers => WorkerWidget::draw(f, app, chunks[1]),
        Tab::Tasks => TaskWidget::draw(f, app, chunks[1]),
        Tab::Queues => QueueWidget::draw(f, app, chunks[1]),
    }

    // Draw status bar
    draw_status_bar(f, app, chunks[2]);

    // Draw help overlay if active
    if app.show_help {
        draw_help(f);
    }

    // Draw confirmation dialog if active
    if app.show_confirmation {
        draw_confirmation_dialog(f, app);
    }

    // Draw task details modal if active
    if app.show_task_details {
        draw_task_details_modal(f, app);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Workers", "Queues", "Tasks"];
    let selected = match app.selected_tab {
        Tab::Workers => 0,
        Tab::Queues => 1,
        Tab::Tasks => 2,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" LazyCelery v0.2.0 "),
        )
        .select(selected)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );

    f.render_widget(tabs, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left side - general info or status message
    let status_left = if !app.status_message.is_empty() {
        app.status_message.clone()
    } else if app.is_searching {
        format!("Search: {}_", app.search_query)
    } else {
        format!(
            "Workers: {} | Tasks: {} | Queues: {}",
            app.workers.len(),
            app.tasks.len(),
            app.queues.len()
        )
    };

    let status_left_widget = Block::default()
        .borders(Borders::ALL)
        .title(Span::raw(status_left));
    f.render_widget(status_left_widget, status_chunks[0]);

    // Right side - key hints
    let key_hints = if app.show_confirmation {
        "[y/Enter] Confirm | [n/Esc] Cancel"
    } else if app.show_task_details {
        "[Any key] Close details"
    } else if app.is_searching {
        "[Enter] Confirm | [Esc] Cancel"
    } else {
        match app.selected_tab {
            Tab::Queues => "[Tab] Switch | [↑↓] Navigate | [p] Purge | [/] Search | [?] Help | [q] Quit",
            Tab::Tasks => "[Tab] Switch | [↑↓] Navigate | [Enter/d] Details | [r] Retry | [x] Revoke | [/] Search | [?] Help | [q] Quit",
            _ => "[Tab] Switch | [↑↓] Navigate | [/] Search | [?] Help | [q] Quit",
        }
    };

    let status_right_widget = Block::default()
        .borders(Borders::ALL)
        .title(Span::raw(key_hints));
    f.render_widget(status_right_widget, status_chunks[1]);
}

fn draw_help(f: &mut Frame) {
    use ratatui::widgets::{Clear, Paragraph, Wrap};

    let area = centered_rect(60, 60, f.size());
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

fn draw_confirmation_dialog(f: &mut Frame, app: &App) {
    use ratatui::widgets::{Clear, Paragraph, Wrap};

    let area = centered_rect(50, 30, f.size());
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_task_details_modal(f: &mut Frame, app: &App) {
    use ratatui::widgets::{Clear, Paragraph, Wrap};

    if let Some(task) = &app.selected_task_details {
        let popup_area = centered_rect(80, 70, f.size());

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
        let details_lines = vec![
            Line::from(vec![
                Span::styled(
                    "ID: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&task.id),
            ]),
            Line::from(vec![
                Span::styled(
                    "Name: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&task.name),
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
                    Style::default().fg(match task.status {
                        crate::models::TaskStatus::Success => Color::Green,
                        crate::models::TaskStatus::Failure => Color::Red,
                        crate::models::TaskStatus::Retry => Color::Yellow,
                        crate::models::TaskStatus::Pending => Color::Blue,
                        crate::models::TaskStatus::Revoked => Color::Magenta,
                        _ => Color::White,
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Worker: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(task.worker.as_deref().unwrap_or("Unknown")),
            ]),
            Line::from(vec![
                Span::styled(
                    "Queue: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("default"),
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
        let mut all_lines = details_lines;
        if task.status == crate::models::TaskStatus::Failure {
            if let Some(traceback) = &task.traceback {
                all_lines.push(Line::from(""));
                all_lines.push(Line::from(vec![Span::styled(
                    "Traceback: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]));
                // Split traceback into lines and add them
                for line in traceback.lines() {
                    all_lines.push(Line::from(Span::styled(
                        line,
                        Style::default().fg(Color::Red),
                    )));
                }
            }
        }

        // Add footer
        all_lines.push(Line::from(""));
        all_lines.push(Line::from(vec![Span::styled(
            "Press any key to close",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        )]));

        let paragraph = Paragraph::new(all_lines)
            .wrap(Wrap { trim: true })
            .scroll((0, 0));

        f.render_widget(paragraph, inner_area);
    }
}
